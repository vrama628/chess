use std::process::ExitCode;

mod game;
mod tui;

use ratatui::crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    ExecutableCommand,
};
use tui::Tui;

fn main() -> ExitCode {
    let mut tui = Tui::new();
    let mut terminal = ratatui::init();
    terminal
        .backend_mut()
        .writer_mut()
        .execute(EnableMouseCapture)
        .expect("enable mouse capture");
    let result = tui.run(&mut terminal);
    terminal
        .backend_mut()
        .writer_mut()
        .execute(DisableMouseCapture)
        .expect("disable mouse capture");
    ratatui::restore();
    match result {
        Ok(outcome) => {
            if let Some(outcome) = outcome {
                println!("Outcome: {outcome}");
            } else {
                println!("Quit before game ended");
            }
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("ERROR: {e}");
            ExitCode::FAILURE
        }
    }
}
