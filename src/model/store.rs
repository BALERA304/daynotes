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
