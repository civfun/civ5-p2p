use civ5_p2p_core::run;
use anyhow::Result;
use iced::{executor, Application, Command, Element, Settings, Text};

fn main() -> Result<()> {
    App::run(Settings::default());

    // Ok(run()?)
    Ok(())
}

struct App;


impl Application for App {
    type Executor = executor::Null;
    type Message = ();
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Self::Message>) {
        (Self, Command::none())
    }

    fn title(&self) -> String {
        String::from("A cool application")
    }

    fn update(&mut self, _message: Self::Message) -> Command<Self::Message> {
        Command::none()
    }

    fn view(&mut self) -> Element<Self::Message> {
        Text::new("Hello, world!").into()
    }
}