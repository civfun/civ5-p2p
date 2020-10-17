use async_std;
use anyhow::Result;
use civ5_p2p_core::{Civ5p2p, Action, Event};
use civ5_p2p_cli::CommandLineInterface;

#[async_std::main]
async fn main() -> Result<()> {
    let keypair = Civ5p2p::new_keypair();
    let p2p = Civ5p2p::new(keypair.clone());
    let (action_tx, event_rx) = p2p.run().await?;

    let mut cli = CommandLineInterface::new(keypair, action_tx, event_rx);
    cli.run().await?;

    Ok(())
}
