use tokio::sync::mpsc;

mod world;
mod server;
mod messages;



#[tokio::main]
async fn main() -> anyhow::Result<()> { 

    let (server_to_world_tx, server_to_world_rx) = mpsc::unbounded_channel();
    let (world_to_server_tx, world_to_server_rx) = mpsc::unbounded_channel();

    let server_handle = tokio::spawn(server::run_server(
        server_to_world_tx.clone(),
        world_to_server_rx,
    ));

    let world_handle = tokio::spawn(world::run_world(
        server_to_world_rx,
        world_to_server_tx.clone(),
    ));

    tokio::try_join!(server_handle, world_handle)?;

    Ok(())
}
