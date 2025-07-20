use std::{fs, net::SocketAddr, sync::Arc};
use h3::server::Connection as H3ServerConn;
use h3_quinn::Connection as H3QuinnConn;
use h3_webtransport::server::WebTransportSession;
use quinn::{Endpoint, ServerConfig};
use tracing::{error, info};
use tracing_subscriber;
use bytes::{Bytes};
use rustls_pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer};
use quinn::crypto::rustls::HandshakeData;
use quinn::crypto::rustls::QuicServerConfig;
use http::{Response, StatusCode};
use h3::server as H3Server;

pub const ALPN_QUIC_HTTP: &[&[u8]] = &[b"h3", b"h3-29"];

fn load_der_cert_and_key() -> anyhow::Result<(Vec<CertificateDer<'static>>, PrivateKeyDer<'static>)> {
    let cert_bytes = fs::read("cert.der")?;
    let key_bytes = fs::read("key.der")?;

    let cert = CertificateDer::from(cert_bytes);
    let pkcs8 = PrivatePkcs8KeyDer::from(key_bytes);
    let key = PrivateKeyDer::from(pkcs8);

    Ok((vec![cert], key))
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    rustls::crypto::CryptoProvider::install_default(
        rustls::crypto::ring::default_provider()
    ).unwrap();

    tracing_subscriber::fmt::init();


    // Load cert and key from files (assumes valid PKCS#8 PEM format)

    let (certs, key) = load_der_cert_and_key()?;

    let mut server_crypto = rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certs, key)?;
    server_crypto.alpn_protocols = ALPN_QUIC_HTTP.iter().map(|&x| x.into()).collect();
    server_crypto.key_log = Arc::new(rustls::KeyLogFile::new());


    let mut server_config = ServerConfig::with_crypto(Arc::new(QuicServerConfig::try_from(server_crypto)?));
    let transport_config = quinn::TransportConfig::default();
    server_config.transport = Arc::new(transport_config);

    // let transport_config = Arc::get_mut(&mut server_config.transport).unwrap();
    // transport_config.max_concurrent_uni_streams(100_u8.into());

    // Bind and start server
    let addr: SocketAddr = "0.0.0.0:8443".parse()?;
    let endpoint = Endpoint::server(server_config, addr)?;
    info!("Listening on https://{}", addr);

    // Accept connections
    while let Some(connecting) = endpoint.accept().await {

        tokio::spawn(async move {
            info!("HERE");
            match connecting.await {
                Ok(quinn_conn) => {
                    let alpn = quinn_conn
                    .handshake_data()
                    .and_then(|data| {
                        data.downcast_ref::<HandshakeData>()
                        .and_then(|hs| hs.protocol.clone())
                    });

                    info!(
                        "🔍 Negotiated ALPN: {:?}",
                        alpn.map(|p| String::from_utf8(p).unwrap_or_else(|_| "<invalid utf8>".to_string()))
                    );

                    let h3_quinn_conn = H3QuinnConn::new(quinn_conn);

                    let mut builder = H3Server::builder();
                    builder
                        .enable_webtransport(true)
                        .enable_extended_connect(true)
                        .enable_datagram(true)
                        .max_webtransport_sessions(100);

                    let h3_conn = builder.build(h3_quinn_conn).await.unwrap();


                    tokio::spawn(handle_h3(h3_conn));
                },
                Err(e) => error!("QUIC handshake failed: {e:?}"),
            }
        });
    }
    Ok(())
}

pub async fn handle_h3(
    mut h3_conn: H3ServerConn<H3QuinnConn, Bytes>,
) -> Result<(), anyhow::Error> {
    info!("Connection");
    while let Some(resolver) = h3_conn.accept().await? {
        let (req, mut stream) = resolver.resolve_request().await?;
        info!("REQ: {:#?}", req);

        if req.method() == http::Method::CONNECT {
            let response = Response::builder()
                .status(StatusCode::OK)
                .header("sec-webtransport-http3-draft", "draft02")
                .version(http::Version::HTTP_3)
                .body(())?;

            info!("RES: {:#?}", response);
            stream.send_response(response).await?;
            stream.finish().await?;
            println!("✅ Handshake response sent");

            // // 🔥 This is the missing piece!
            let session = WebTransportSession::accept(req, stream, h3_conn).await?;
            // println!("WebTransport session established");

            // // Send test datagram to confirm
            // session.send_datagram(Bytes::from_static(b"hello from server"))?;
            // println!("Sent datagram");

            // // Optionally receive datagram
            // tokio::spawn(async move {
            //     while let Some(Ok(data)) = session.read_datagram().await {
            //         println!("Got datagram from client: {:?}", data);
            //     }
            // });
        } else {
            let response = Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body(())?;
            stream.send_response(response).await?;
            stream.finish().await?;
            println!("Non-CONNECT request rejected");
        }
    }
    Ok(())
}