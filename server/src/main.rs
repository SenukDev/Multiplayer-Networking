use anyhow::{Context, Result};
use bytes::Bytes;
use h3::{
    ext::Protocol,
    quic::{self},
    server::Connection,
};
use h3_quinn::quinn::{self, crypto::rustls::QuicServerConfig};
use h3_webtransport::server::{self, WebTransportSession};
use http::Method;
use rustls::pki_types::{CertificateDer, PrivateKeyDer, ServerName, UnixTime};
use std::{net::SocketAddr, path::PathBuf, sync::Arc, time::Duration};
use structopt::StructOpt;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio::pin;
use tracing::{error, info, trace_span};
use rustls::DigitallySignedStruct;
use rustls::SignatureScheme;
use rustls::client::danger::{ServerCertVerifier, ServerCertVerified, HandshakeSignatureValid};

#[derive(StructOpt, Debug)]
#[structopt(name = "server")]
struct Opt {
    #[structopt(
        short,
        long,
        default_value = "[::]:4433",
        help = "What address:port to listen for new connections"
    )]
    pub listen: SocketAddr,

    #[structopt(flatten)]
    pub certs: Certs,
}

#[derive(StructOpt, Debug)]
pub struct Certs {
    #[structopt(
        long,
        short,
        default_value = "localhost.crt.der",
        help = "Certificate for TLS. If present, `--key` is mandatory."
    )]
    pub cert: PathBuf,

    #[structopt(
        long,
        short,
        default_value = "localhost.key.der",
        help = "Private key for the certificate."
    )]
    pub key: PathBuf,
}

#[tokio::main]
async fn main() -> Result<()> {
    // 🔐 Required for rustls 0.23+
    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .expect("Failed to install crypto provider");

    // 📋 Logging/tracing
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_span_events(tracing_subscriber::fmt::format::FmtSpan::FULL)
        .with_writer(std::io::stderr)
        .init();

    let opt = Opt::from_args();
    tracing::info!("Opt: {opt:#?}");
    let Certs { cert, key } = opt.certs;

    let cert = CertificateDer::from(std::fs::read(cert).context("Failed to read cert")?);
    let key = PrivateKeyDer::try_from(std::fs::read(key).context("Failed to read key")?)
        .expect("Invalid DER private key");

    let mut tls_config = rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(vec![cert], key)?;

    tls_config.max_early_data_size = u32::MAX;
    tls_config.alpn_protocols = vec![
        b"h3".to_vec(),
        b"h3-32".to_vec(),
        b"h3-31".to_vec(),
        b"h3-30".to_vec(),
        b"h3-29".to_vec(),
    ];

    let mut server_config =
        quinn::ServerConfig::with_crypto(Arc::new(QuicServerConfig::try_from(tls_config)?));

    let mut transport_config = quinn::TransportConfig::default();
    transport_config.keep_alive_interval(Some(Duration::from_secs(2)));
    server_config.transport = Arc::new(transport_config);

    let endpoint = quinn::Endpoint::server(server_config, opt.listen)?;
    info!("listening on {}", opt.listen);

    while let Some(new_conn) = endpoint.accept().await {
        tokio::spawn(async move {
            match new_conn.await {
                Ok(conn) => {
                    info!("new http3 connection");
                    let h3_conn = h3::server::builder()
                        .enable_webtransport(true)
                        .enable_extended_connect(true)
                        .enable_datagram(true)
                        .max_webtransport_sessions(1)
                        .send_grease(true)
                        .build(h3_quinn::Connection::new(conn))
                        .await
                        .unwrap();

                    if let Err(err) = handle_connection(h3_conn).await {
                        error!("Failed to handle connection: {err:?}");
                    }
                }
                Err(err) => {
                    error!("Connection failed: {err:?}");
                }
            }
        });
    }

    endpoint.wait_idle().await;
    Ok(())
}

async fn handle_connection(mut conn: Connection<h3_quinn::Connection, Bytes>) -> Result<()> {
    loop {
        match conn.accept().await {
            Ok(Some(resolver)) => {
                let (req, stream) = match resolver.resolve_request().await {
                    Ok(request) => request,
                    Err(err) => {
                        error!("error resolving request: {err:?}");
                        continue;
                    }
                };

                info!("new request: {:#?}", req);

                if req.method() == Method::CONNECT
                    && req
                        .extensions()
                        .get::<Protocol>()
                        == Some(&Protocol::WEB_TRANSPORT)
                {
                    tracing::info!("Peer wants WebTransport");
                    let session = WebTransportSession::accept(req, stream, conn).await?;
                    tracing::info!("WebTransport session established");

                    handle_session_and_echo_all_inbound_messages(session).await?;
                    return Ok(());
                } else {
                    tracing::info!(?req, "Received non-WebTransport request");
                }
            }
            Ok(None) => break, // connection closed
            Err(err) => {
                error!("Connection errored: {err}");
                break;
            }
        }
    }

    Ok(())
}

macro_rules! log_result {
    ($expr:expr) => {
        if let Err(err) = $expr {
            tracing::error!("{err:?}");
        }
    };
}

async fn echo_stream<T, R>(send: T, recv: R) -> Result<()>
where
    T: AsyncWrite,
    R: AsyncRead,
{
    pin!(send);
    pin!(recv);

    let mut buf = Vec::new();
    recv.read_to_end(&mut buf).await?;
    let message = Bytes::from(buf);
    send_chunked(send, message).await?;
    Ok(())
}

async fn send_chunked(mut send: impl AsyncWrite + Unpin, data: Bytes) -> Result<()> {
    for chunk in data.chunks(4) {
        tokio::time::sleep(Duration::from_millis(100)).await;
        tracing::info!("Sending {chunk:?}");
        send.write_all(chunk).await?;
    }
    Ok(())
}

async fn open_bidi_test<S>(mut stream: S) -> Result<()>
where
    S: Unpin + AsyncRead + AsyncWrite,
{
    stream
        .write_all(b"Hello from server bidi stream")
        .await
        .context("Failed to respond")?;

    let mut resp = Vec::new();
    stream.shutdown().await?;
    stream.read_to_end(&mut resp).await?;
    tracing::info!("Client response: {resp:?}");
    Ok(())
}

async fn handle_session_and_echo_all_inbound_messages(
    session: WebTransportSession<h3_quinn::Connection, Bytes>,
) -> Result<()> {
    let session_id = session.session_id();

    let stream = session.open_bi(session_id).await?;
    tokio::spawn(async move { log_result!(open_bidi_test(stream).await) });

    let mut datagram_reader = session.datagram_reader();
    let mut datagram_sender = session.datagram_sender();

    loop {
        tokio::select! {
            datagram = datagram_reader.read_datagram() => {
                let datagram = match datagram {
                    Ok(d) => d,
                    Err(err) => {
                        tracing::error!("Datagram read error: {err:?}");
                        break;
                    }
                };
                tracing::info!("Received datagram: {datagram:?}");
                let payload = datagram.into_payload();
                datagram_sender.send_datagram(payload)?;
            }

            uni_stream = session.accept_uni() => {
                let (id, stream) = uni_stream?.unwrap();
                let send = session.open_uni(id).await?;
                tokio::spawn(async move { log_result!(echo_stream(send, stream).await); });
            }

            bi_stream = session.accept_bi() => {
                if let Some(server::AcceptedBi::BidiStream(_, stream)) = bi_stream? {
                    let (send, recv) = quic::BidiStream::split(stream);
                    tokio::spawn(async move { log_result!(echo_stream(send, recv).await); });
                }
            }

            else => break,
        }
    }

    tracing::info!("Session finished");
    Ok(())
}
