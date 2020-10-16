use rustyline::error::ReadlineError;
use rustyline::Editor;
use civ5_p2p_core::Civ5p2p;
use async_std;

#[async_std::main]
async fn main() {
    let keypair = Civ5p2p::new_keypair();
    let p2p = Civ5p2p::new(keypair);
    let a = p2p.run().await;

    let mut rl = Editor::<()>::new();
    if rl.load_history("history.txt").is_err() {
        println!("No previous history.");
    }
    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str());
                println!("Line: {}", line);
            },
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break
            },
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break
            },
            Err(err) => {
                println!("Error: {:?}", err);
                break
            }
        }
    }
    rl.save_history("history.txt").unwrap();
}
