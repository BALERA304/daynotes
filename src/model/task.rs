use serde::{Deserialize, Serialize};

/// Приоритет задачи или направления.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Priority {
    Low,
    Med,
    High,
}

impl Priority {
    /// Числовой вес для сортировки: чем больше, тем важнее.
    pub fn weight(self) -> u8 {
        match self {
            Priority::High => 3,
            Priority::Med => 2,
            Priority::Low => 1,
        }
    }

    /// Отображаемое имя.
    pub fn label(self) -> &'static str {
        match self {
            Priority::High => "high",
            Priority::Med => "med",
            Priority::Low => "low",
        }
    }

    /// Следующий приоритет по кругу (для TUI-формы).
    pub fn next(self) -> Self {
        match self {
            Priority::Low => Priority::Med,
            Priority::Med => Priority::High,
            Priority::High => Priority::Low,
        }
    }

    /// Предыдущий приоритет по кругу.
    pub fn prev(self) -> Self {
        match self {
            Priority::Low => Priority::High,
            Priority::High => Priority::Med,
            Priority::Med => Priority::Low,
        }
    }
}

impl Default for Priority {
    fn default() -> Self {
        Priority::Med
    }
}

// =====================================

/// Задача на конкретный день.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    /// Короткий id (8 символов).
    pub id: String,

    /// Название задачи.
    pub title: String,

    /// Дополнительное описание.
    #[serde(default)]
    pub description: String,

    /// Приоритет.
    #[serde(default)]
    pub priority: Priority,

    /// Веха (2–10 слов).
    #[serde(default)]
    pub milestone: String,

    /// Комментарий. Пусто = не сделано.
    #[serde(default)]
    pub comment: String,
}

impl Task {
    /// Создать новую задачу с сгенерированным id.
    pub fn new(title: String) -> Self {
        Self {
            id: crate::model::date::generate_id(),
            title,
            description: String::new(),
            priority: Priority::default(),
            milestone: String::new(),
            comment: String::new(),
        }
    }

    /// Выполнена ли задача (комментарий не пуст).
    pub fn is_done(&self) -> bool {
        !self.comment.trim().is_empty()
    }
}
