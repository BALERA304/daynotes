mod cli;
mod controller;
mod model;
mod view;

use clap::Parser;
use cli::{Cli, Commands, DirectionCommands};

fn main() {
    let cli = Cli::parse();

    match cli.command {
        None => {
            println!("[dn] открыть сегодня — заглушка");
        }
        Some(Commands::Next) => {
            println!("[dn next] открыть завтра — заглушка");
        }
        Some(Commands::Past) => {
            println!("[dn past] открыть вчера — заглушка");
        }
        Some(Commands::Open { date }) => {
            println!("[dn open {}] — заглушка", date);
        }
        Some(Commands::Delete { date }) => {
            println!("[dn delete {}] — заглушка", date);
        }
        Some(Commands::Stats) => {
            println!("[dn stats] — заглушка");
        }
        Some(Commands::Direction(cmd)) => match cmd {
            DirectionCommands::Add => {
                println!("[dn direction add] — заглушка");
            }
            DirectionCommands::Edit => {
                println!("[dn direction edit] — заглушка");
            }
            DirectionCommands::Delete { name } => {
                println!("[dn direction delete {}] — заглушка", name);
            }
        },
    }
}
