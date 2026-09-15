use serde::{Deserialize, Serialize};

use super::task::Priority;

/// Жизненное направление (например, "Здоровье", "Карьера").
///
/// Направления живут отдельно от задач и дней. Они нужны
/// как ориентир: на что я вообще трачу силы.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Direction {
    /// Короткий id (8 символов).
    pub id: String,

    /// Название направления.
    pub name: String,

    /// Описание (1–6 предложений).
    #[serde(default)]
    pub description: String,

    /// Приоритет.
    #[serde(default)]
    pub priority: Priority,
}

impl Direction {
    /// Создать новое направление с сгенерированным id.
    pub fn new(name: String) -> Self {
        Self {
            id: crate::model::date::generate_id(),
            name,
            description: String::new(),
            priority: Priority::default(),
        }
    }
}
