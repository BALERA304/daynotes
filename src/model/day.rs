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
    #[allow(dead_code)]
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
    #[allow(dead_code)]
    pub fn all_done(&self) -> bool {
        self.tasks.iter().all(|t| t.is_done())
    }

    /// Сколько задач выполнено.

    #[allow(dead_code)]
    pub fn done_count(&self) -> usize {
        self.tasks.iter().filter(|t| t.is_done()).count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::task::Priority;

    /// Вспомогательная: создать задачу с заданным приоритетом.
    fn task_with_priority(title: &str, priority: Priority) -> Task {
        let mut t = Task::new(title.to_string());
        t.priority = priority;
        t
    }

    #[test]
    fn test_day_new_empty() {
        let day = Day::new();
        assert!(day.tasks.is_empty());
    }

    #[test]
    fn test_day_default_empty() {
        let day = Day::default();
        assert!(day.tasks.is_empty());
    }

    #[test]
    fn test_add_task() {
        let mut day = Day::new();
        let task = Task::new("Тест".to_string());
        let id = task.id.clone();

        day.add_task(task);

        assert_eq!(day.tasks.len(), 1);
        assert_eq!(day.tasks[0].id, id);
    }

    #[test]
    fn test_add_task_sorts_by_priority() {
        let mut day = Day::new();
        day.add_task(task_with_priority("low", Priority::Low));
        day.add_task(task_with_priority("high", Priority::High));
        day.add_task(task_with_priority("med", Priority::Med));

        assert_eq!(day.tasks[0].title, "high");
        assert_eq!(day.tasks[1].title, "med");
        assert_eq!(day.tasks[2].title, "low");
    }

    #[test]
    fn test_sort_tasks_stable_within_priority() {
        // Задачи с одинаковым приоритетом сохраняют порядок добавления.
        let mut day = Day::new();
        day.add_task(task_with_priority("first", Priority::Med));
        day.add_task(task_with_priority("second", Priority::Med));
        day.add_task(task_with_priority("third", Priority::Med));

        // sort_by в Rust стабильная — порядок сохраняется.
        assert_eq!(day.tasks[0].title, "first");
        assert_eq!(day.tasks[1].title, "second");
        assert_eq!(day.tasks[2].title, "third");
    }

    #[test]
    fn test_sort_tasks_after_change() {
        // Если поменять приоритет и вызвать sort_tasks — порядок обновится.
        let mut day = Day::new();
        day.add_task(task_with_priority("a", Priority::Low));
        day.add_task(task_with_priority("b", Priority::Low));

        // Меняем приоритет у "a" на High.
        day.tasks[0].priority = Priority::High;
        day.sort_tasks();

        assert_eq!(day.tasks[0].title, "a");
        assert_eq!(day.tasks[1].title, "b");
    }

    #[test]
    fn test_find_task() {
        let mut day = Day::new();
        let task = Task::new("Найти".to_string());
        let id = task.id.clone();
        day.add_task(task);

        assert!(day.find_task(&id).is_some());
        assert!(day.find_task("nonexistent").is_none());
    }

    #[test]
    fn test_find_task_mut() {
        let mut day = Day::new();
        let task = Task::new("Изменить".to_string());
        let id = task.id.clone();
        day.add_task(task);

        let found = day.find_task_mut(&id).unwrap();
        found.title = "Изменено".to_string();

        assert_eq!(day.find_task(&id).unwrap().title, "Изменено");
    }

    #[test]
    fn test_remove_task() {
        let mut day = Day::new();
        let task = Task::new("Удалить".to_string());
        let id = task.id.clone();
        day.add_task(task);

        assert!(day.remove_task(&id));
        assert!(day.tasks.is_empty());
    }

    #[test]
    fn test_remove_task_not_found() {
        let mut day = Day::new();
        assert!(!day.remove_task("nonexistent"));
    }

    #[test]
    fn test_remove_task_only_matching() {
        let mut day = Day::new();
        let t1 = Task::new("A".to_string());
        let t2 = Task::new("B".to_string());
        let id1 = t1.id.clone();
        let id2 = t2.id.clone();

        day.add_task(t1);
        day.add_task(t2);
        day.remove_task(&id1);

        assert_eq!(day.tasks.len(), 1);
        assert_eq!(day.tasks[0].id, id2);
    }

    #[test]
    fn test_all_done_empty() {
        // Пустой день — все задачи выполнены (vacuously true).
        let day = Day::new();
        assert!(day.all_done());
    }

    #[test]
    fn test_all_done_false() {
        let mut day = Day::new();
        day.add_task(Task::new("A".to_string()));
        assert!(!day.all_done());
    }

    #[test]
    fn test_all_done_true() {
        let mut day = Day::new();
        let mut t1 = Task::new("A".to_string());
        t1.comment = "готово".to_string();
        let mut t2 = Task::new("B".to_string());
        t2.comment = "тоже готово".to_string();

        day.add_task(t1);
        day.add_task(t2);
        assert!(day.all_done());
    }

    #[test]
    fn test_done_count_empty() {
        let day = Day::new();
        assert_eq!(day.done_count(), 0);
    }

    #[test]
    fn test_done_count_mixed() {
        let mut day = Day::new();

        let mut t1 = Task::new("A".to_string());
        t1.comment = "готово".to_string();

        let t2 = Task::new("B".to_string());

        let mut t3 = Task::new("C".to_string());
        t3.comment = "готово".to_string();

        day.add_task(t1);
        day.add_task(t2);
        day.add_task(t3);

        assert_eq!(day.done_count(), 2);
    }

    #[test]
    fn test_serde_roundtrip() {
        let mut day = Day::new();
        day.add_task(Task::new("A".to_string()));
        day.add_task(Task::new("B".to_string()));

        let json = serde_json::to_string(&day).unwrap();
        let parsed: Day = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.tasks.len(), 2);
    }

    #[test]
    fn test_serde_empty() {
        // Пустой JSON должен десериализоваться в день без задач.
        let json = "{}";
        let day: Day = serde_json::from_str(json).unwrap();
        assert!(day.tasks.is_empty());
    }
}
