mod cli;
mod controller;
mod model;
mod view;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, Commands, DirectionCommands};

fn main() -> Result<()> {
    // Проверка бэкапа и восстановление — до всего остального.
    check_and_restore()?;

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
            DirectionCommands::Delete { name } => {
                controller::actions::delete_direction(&name)?;
                println!("удалено направление: {}", name);
            }
        },
    }

    Ok(())
}

/// Проверить целостность JSON. Если битый — предложить восстановить из бэкапа.
fn check_and_restore() -> Result<()> {
    use model::storage;

    // Если файла нет — ничего не делаем, storage::load() создаст новый.
    if !storage::file_path()?.exists() {
        return Ok(());
    }

    // Если файл валидный — ок.
    if !storage::is_corrupted()? {
        return Ok(());
    }

    // Файл битый.
    eprintln!("Ошибка: файл ~/.dn/day_notes.json повреждён.");
    eprintln!();

    let backup = storage::backup_path()?;
    if backup.exists() {
        eprintln!("Найден бэкап: {:?}", backup);
        eprintln!("Восстановить из бэкапа? (y/n)");

        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;

        if input.trim().eq_ignore_ascii_case("y") {
            storage::restore_from_backup()?;
            eprintln!("Восстановлено из бэкапа.");
            return Ok(());
        }
    } else {
        eprintln!("Бэкап не найден.");
    }

    eprintln!("Создать новый пустой файл? (y/n)");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;

    if input.trim().eq_ignore_ascii_case("y") {
        storage::create_new()?;
        eprintln!("Создан новый пустой файл.");
        Ok(())
    } else {
        anyhow::bail!("отменено пользователем");
    }
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
