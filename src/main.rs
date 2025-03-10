use std::process::ExitCode;

mod ai;
mod game;
mod tui;

use clap::Parser;
use ratatui::crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    ExecutableCommand,
};
use tui::Tui;

#[derive(Parser)]
struct Args {
    ai: Option<usize>,
}

fn main() -> ExitCode {
    let Args { ai } = Args::parse();
    let mut tui = Tui::new(ai);
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
