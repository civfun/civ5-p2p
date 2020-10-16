use rustyline::error::ReadlineError;
use rustyline::Editor;
use civ5_p2p_core::{Civ5p2p, Action, Event};
use futures::channel::mpsc;
use anyhow::Result;
use libp2p::identity::Keypair;

pub struct CommandLineInterface {
    keypair: Keypair,
    action_tx: mpsc::UnboundedSender<Action>,
    event_rx: mpsc::UnboundedReceiver<Event>,
}

impl CommandLineInterface {
    pub fn new(keypair: Keypair, action_tx: mpsc::UnboundedSender<Action>, event_rx: mpsc::UnboundedReceiver<Event>) -> Self {
        Self { keypair, action_tx, event_rx }
    }

    pub async fn run(&self) -> Result<()> {
        let mut rl = Editor::<()>::new();
        // if rl.load_history("history.txt").is_err() {
        //     println!("No previous history.");
        // }
        loop {
            let readline = rl.readline(">> ");
            match readline {
                Ok(line) => {
                    println!("Line: {}", line);
                    // rl.add_history_entry(line.as_str());
                    self.handle_cmd(&line).await?;
                }
                Err(ReadlineError::Interrupted) => {
                    println!("CTRL-C");
                    break;
                }
                Err(ReadlineError::Eof) => {
                    println!("CTRL-D");
                    break;
                }
                Err(err) => {
                    println!("Error: {:?}", err);
                    break;
                }
            }
        }
        // rl.save_history("history.txt").unwrap();

        Ok(())
    }

    async fn handle_cmd(&self, line: &str) -> Result<()> {
        Ok(())
    }
}
