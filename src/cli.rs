use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "dn")]
#[command(about = "Day Notes — локальный ежедневник", long_about = None)]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Открыть завтра
    Next,

    /// Открыть вчера
    Past,

    /// Открыть конкретную дату
    Open {
        /// Дата в формате дд-мм-гггг
        date: String,
    },

    /// Удалить день
    Delete {
        /// Дата в формате дд-мм-гггг
        date: String,
    },

    /// Статистика
    Stats,

    /// Управление направлениями
    #[command(subcommand)]
    Direction(DirectionCommands),
}

#[derive(Subcommand)]
pub enum DirectionCommands {
    /// Удалить направление
    Delete {
        /// Название направления
        name: String,
    },
}
