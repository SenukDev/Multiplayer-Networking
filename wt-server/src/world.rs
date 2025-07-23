use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio::time::{interval, Duration};
use anyhow::Result;
use crate::messages::{ServerToWorld, WorldToServer};


pub async fn run_world(
    from_server: UnboundedReceiver<ServerToWorld>,
    mut to_server: UnboundedSender<WorldToServer>,
) -> Result<()> {
    let mut tick = interval(Duration::from_secs(1));
    let mut tick_count = 0;

    loop {
        tick.tick().await;
        tick_count += 1;
        println!("World Here! {} ", tick_count);
    }
}