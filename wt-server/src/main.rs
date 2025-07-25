use tokio::sync::mpsc;

mod world;
mod server;
mod messages;
mod components;
mod systems;
mod network;
mod scripts;


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

    let (server_result, world_result) = tokio::try_join!(server_handle, world_handle)?;
    println!("Server finished: {:?}", server_result);
    println!("World finished: {:?}", world_result);

    Ok(())
}
