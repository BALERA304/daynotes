mod cli;
mod controller;
mod model;
mod view;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, Commands, DirectionCommands};

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        None => {
            let (store, date) = controller::actions::open_today()?;
            view::tui::run(store, date)?;
        }
        Some(Commands::Next) => {
            let (store, date) = controller::actions::open_next()?;
            view::tui::run(store, date)?;
        }
        Some(Commands::Past) => {
            let (store, date) = controller::actions::open_past()?;
            view::tui::run(store, date)?;
        }
        Some(Commands::Open { date }) => {
            let (store, d) = controller::actions::open_date(&date)?;
            view::tui::run(store, d)?;
        }
        Some(Commands::Delete { date }) => {
            controller::actions::delete_day(&date)?;
            println!("удалён день: {}", date);
        }
        Some(Commands::Stats) => {
            let stats = controller::actions::collect_stats()?;
            print_stats(&stats);
        }
        Some(Commands::Direction(cmd)) => match cmd {
            DirectionCommands::Add => {
                println!("[dn direction add] — TUI-форма будет в Эпике 4");
            }
            DirectionCommands::Edit => {
                println!("[dn direction edit] — TUI-форма будет в Эпике 4");
            }
            DirectionCommands::Delete { name } => {
                controller::actions::delete_direction(&name)?;
                println!("удалено направление: {}", name);
            }
        },
    }

    Ok(())
}

fn print_stats(stats: &controller::actions::Stats) {
    println!("всего задач: {}", stats.total_tasks);
    let percent = if stats.total_tasks > 0 {
        (stats.done_tasks * 100) / stats.total_tasks
    } else {
        0
    };
    println!("выполнено: {} ({}%)", stats.done_tasks, percent);
    println!();
    println!("за неделю: {}", stats.done_this_week);
    println!("за месяц:");
    for (month, count) in &stats.by_month {
        // month — "мм-гггг", например "09-2026"
        let parts: Vec<&str> = month.split('-').collect();
        if parts.len() == 2 {
            let month_num = parts[0];
            let year_short = &parts[1][2..];
            let month_name = month_name_ru(month_num);
            println!("  {} {}: {}", month_name, year_short, count);
        }
    }
}

fn month_name_ru(num: &str) -> &'static str {
    match num {
        "01" => "январь",
        "02" => "февраль",
        "03" => "март",
        "04" => "апрель",
        "05" => "май",
        "06" => "июнь",
        "07" => "июль",
        "08" => "август",
        "09" => "сентябрь",
        "10" => "октябрь",
        "11" => "ноябрь",
        "12" => "декабрь",
        _ => "?",
    }
}
