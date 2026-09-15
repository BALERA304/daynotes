use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::day::Day;
use super::direction::Direction;

/// Текущая версия формата JSON.
pub const CURRENT_VERSION: u32 = 1;

/// Корневая структура всего файла `day_notes.json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Store {
    /// Версия формата (для будущих миграций).
    pub version: u32,

    /// Жизненные направления.
    #[serde(default)]
    pub directions: Vec<Direction>,

    /// Дни, ключ — дата в формате `дд-мм-гггг`.
    ///
    /// `BTreeMap` — чтобы ключи в JSON всегда были отсортированы.
    #[serde(default)]
    pub days: BTreeMap<String, Day>,
}

impl Default for Store {
    fn default() -> Self {
        Self {
            version: CURRENT_VERSION,
            directions: Vec::new(),
            days: BTreeMap::new(),
        }
    }
}

impl Store {
    /// Создать пустой Store.
    pub fn new() -> Self {
        Self::default()
    }

    // --- Дни ---

    /// Получить день (если есть).
    pub fn get_day(&self, date: &str) -> Option<&Day> {
        self.days.get(date)
    }

    /// Получить день (мутабельно).
    pub fn get_day_mut(&mut self, date: &str) -> Option<&mut Day> {
        self.days.get_mut(date)
    }

    /// Есть ли день.
    pub fn has_day(&self, date: &str) -> bool {
        self.days.contains_key(date)
    }

    /// Создать день, если его нет. Возвращает true, если создали.
    pub fn ensure_day(&mut self, date: &str) -> bool {
        if self.days.contains_key(date) {
            false
        } else {
            self.days.insert(date.to_string(), Day::new());
            true
        }
    }

    /// Удалить день. Возвращает true, если что-то удалили.
    pub fn remove_day(&mut self, date: &str) -> bool {
        self.days.remove(date).is_some()
    }

    // --- Направления ---

    /// Добавить направление.
    pub fn add_direction(&mut self, direction: Direction) {
        self.directions.push(direction);
        self.sort_directions();
    }

    /// Отсортировать направления по приоритету.
    pub fn sort_directions(&mut self) {
        self.directions
            .sort_by(|a, b| b.priority.weight().cmp(&a.priority.weight()));
    }

    /// Найти направление по имени.
    #[allow(dead_code)]
    pub fn find_direction(&self, name: &str) -> Option<&Direction> {
        self.directions.iter().find(|d| d.name == name)
    }

    /// Найти направление по имени (мутабельно).
    pub fn find_direction_mut(&mut self, name: &str) -> Option<&mut Direction> {
        self.directions.iter_mut().find(|d| d.name == name)
    }

    /// Удалить направление по имени. Возвращает true, если что-то удалили.
    pub fn remove_direction(&mut self, name: &str) -> bool {
        let len_before = self.directions.len();
        self.directions.retain(|d| d.name != name);
        self.directions.len() != len_before
    }

    /// Есть ли направление с таким именем.
    pub fn has_direction(&self, name: &str) -> bool {
        self.directions.iter().any(|d| d.name == name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::task::Priority;

    fn dir(name: &str, priority: Priority) -> Direction {
        let mut d = Direction::new(name.to_string());
        d.priority = priority;
        d
    }

    // --- Дни ---

    #[test]
    fn test_store_new_empty() {
        let store = Store::new();
        assert_eq!(store.version, CURRENT_VERSION);
        assert!(store.directions.is_empty());
        assert!(store.days.is_empty());
    }

    #[test]
    fn test_store_default() {
        let store = Store::default();
        assert_eq!(store.version, CURRENT_VERSION);
    }

    #[test]
    fn test_has_day() {
        let mut store = Store::new();
        assert!(!store.has_day("14-09-2026"));

        store.ensure_day("14-09-2026");
        assert!(store.has_day("14-09-2026"));
        assert!(!store.has_day("15-09-2026"));
    }

    #[test]
    fn test_ensure_day_creates() {
        let mut store = Store::new();
        let created = store.ensure_day("14-09-2026");
        assert!(created);
        assert!(store.has_day("14-09-2026"));
    }

    #[test]
    fn test_ensure_day_existing() {
        let mut store = Store::new();
        store.ensure_day("14-09-2026");

        let created = store.ensure_day("14-09-2026");
        assert!(!created);
    }

    #[test]
    fn test_ensure_day_doesnt_overwrite() {
        let mut store = Store::new();
        store.ensure_day("14-09-2026");

        // Добавим задачу.
        let task = crate::model::task::Task::new("A".to_string());
        store.get_day_mut("14-09-2026").unwrap().add_task(task);

        // Повторный ensure_day не должен стирать задачи.
        store.ensure_day("14-09-2026");

        assert_eq!(store.get_day("14-09-2026").unwrap().tasks.len(), 1);
    }

    #[test]
    fn test_get_day_missing() {
        let store = Store::new();
        assert!(store.get_day("14-09-2026").is_none());
    }

    #[test]
    fn test_remove_day() {
        let mut store = Store::new();
        store.ensure_day("14-09-2026");

        assert!(store.remove_day("14-09-2026"));
        assert!(!store.has_day("14-09-2026"));
    }

    #[test]
    fn test_remove_day_missing() {
        let mut store = Store::new();
        assert!(!store.remove_day("14-09-2026"));
    }

    #[test]
    fn test_days_sorted() {
        // BTreeMap хранит ключи отсортированными.
        let mut store = Store::new();
        store.ensure_day("20-09-2026");
        store.ensure_day("14-09-2026");
        store.ensure_day("18-09-2026");

        let keys: Vec<&String> = store.days.keys().collect();
        // Строковая сортировка: 14 < 18 < 20.
        assert_eq!(keys[0], "14-09-2026");
        assert_eq!(keys[1], "18-09-2026");
        assert_eq!(keys[2], "20-09-2026");
    }

    // --- Направления ---

    #[test]
    fn test_add_direction() {
        let mut store = Store::new();
        store.add_direction(dir("Здоровье", Priority::Med));

        assert_eq!(store.directions.len(), 1);
        assert_eq!(store.directions[0].name, "Здоровье");
    }

    #[test]
    fn test_add_direction_sorts() {
        let mut store = Store::new();
        store.add_direction(dir("low", Priority::Low));
        store.add_direction(dir("high", Priority::High));
        store.add_direction(dir("med", Priority::Med));

        assert_eq!(store.directions[0].name, "high");
        assert_eq!(store.directions[1].name, "med");
        assert_eq!(store.directions[2].name, "low");
    }

    #[test]
    fn test_has_direction() {
        let mut store = Store::new();
        assert!(!store.has_direction("Здоровье"));

        store.add_direction(dir("Здоровье", Priority::Med));
        assert!(store.has_direction("Здоровье"));
        assert!(!store.has_direction("Карьера"));
    }

    #[test]
    fn test_find_direction() {
        let mut store = Store::new();
        store.add_direction(dir("Здоровье", Priority::Med));

        assert!(store.find_direction("Здоровье").is_some());
        assert!(store.find_direction("Карьера").is_none());
    }

    #[test]
    fn test_find_direction_mut() {
        let mut store = Store::new();
        store.add_direction(dir("Здоровье", Priority::Med));

        let d = store.find_direction_mut("Здоровье").unwrap();
        d.description = "Новое описание".to_string();

        assert_eq!(
            store.find_direction("Здоровье").unwrap().description,
            "Новое описание"
        );
    }

    #[test]
    fn test_remove_direction() {
        let mut store = Store::new();
        store.add_direction(dir("Здоровье", Priority::Med));

        assert!(store.remove_direction("Здоровье"));
        assert!(!store.has_direction("Здоровье"));
    }

    #[test]
    fn test_remove_direction_missing() {
        let mut store = Store::new();
        assert!(!store.remove_direction("Здоровье"));
    }

    #[test]
    fn test_remove_direction_only_matching() {
        let mut store = Store::new();
        store.add_direction(dir("Здоровье", Priority::Med));
        store.add_direction(dir("Карьера", Priority::Med));

        store.remove_direction("Здоровье");

        assert_eq!(store.directions.len(), 1);
        assert_eq!(store.directions[0].name, "Карьера");
    }

    // --- Сериализация ---

    #[test]
    fn test_serde_roundtrip() {
        let mut store = Store::new();
        store.add_direction(dir("Здоровье", Priority::High));
        store.ensure_day("14-09-2026");
        store
            .get_day_mut("14-09-2026")
            .unwrap()
            .add_task(crate::model::task::Task::new("Задача".to_string()));

        let json = serde_json::to_string(&store).unwrap();
        let parsed: Store = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.version, CURRENT_VERSION);
        assert_eq!(parsed.directions.len(), 1);
        assert_eq!(parsed.days.len(), 1);
    }

    #[test]
    fn test_serde_empty() {
        let json = r#"{"version":1}"#;
        let store: Store = serde_json::from_str(json).unwrap();
        assert!(store.directions.is_empty());
        assert!(store.days.is_empty());
    }

    #[test]
    fn test_serde_version_preserved() {
        let mut store = Store::new();
        store.version = 1;

        let json = serde_json::to_string(&store).unwrap();
        assert!(json.contains("\"version\":1"));
    }
}
