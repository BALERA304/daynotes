use serde::{Deserialize, Serialize};

use super::task::Task;

/// Один день ежедневника.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Day {
    /// Задачи на этот день.
    #[serde(default)]
    pub tasks: Vec<Task>,
}

impl Day {
    /// Создать пустой день.
    pub fn new() -> Self {
        Self { tasks: Vec::new() }
    }

    /// Добавить задачу.
    pub fn add_task(&mut self, task: Task) {
        self.tasks.push(task);
        self.sort_tasks();
    }

    /// Отсортировать задачи по приоритету (high → med → low).
    pub fn sort_tasks(&mut self) {
        self.tasks
            .sort_by(|a, b| b.priority.weight().cmp(&a.priority.weight()));
    }

    /// Найти задачу по id.
    pub fn find_task(&self, id: &str) -> Option<&Task> {
        self.tasks.iter().find(|t| t.id == id)
    }

    /// Найти задачу по id (мутабельно).
    pub fn find_task_mut(&mut self, id: &str) -> Option<&mut Task> {
        self.tasks.iter_mut().find(|t| t.id == id)
    }

    /// Удалить задачу по id. Возвращает true, если что-то удалили.
    pub fn remove_task(&mut self, id: &str) -> bool {
        let len_before = self.tasks.len();
        self.tasks.retain(|t| t.id != id);
        self.tasks.len() != len_before
    }

    /// Все ли задачи выполнены.
    pub fn all_done(&self) -> bool {
        self.tasks.iter().all(|t| t.is_done())
    }

    /// Сколько задач выполнено.
    pub fn done_count(&self) -> usize {
        self.tasks.iter().filter(|t| t.is_done()).count()
    }
}
