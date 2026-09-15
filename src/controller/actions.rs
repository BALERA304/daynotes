use anyhow::{Result, bail};

use crate::model::date;
use crate::model::storage;
use crate::model::store::Store;
use crate::model::task::Task;

/// Открыть сегодня. Создаёт день, если нет.
///
/// Возвращает Store и дату.
pub fn open_today() -> Result<(Store, String)> {
    let mut store = load_store()?;
    let today = date::today();
    store.ensure_day(&today);
    storage::save(&store)?;
    Ok((store, today))
}

/// Открыть завтра. Создаёт день, если нет.
pub fn open_next() -> Result<(Store, String)> {
    let mut store = load_store()?;
    let tomorrow = date::tomorrow();
    store.ensure_day(&tomorrow);
    storage::save(&store)?;
    Ok((store, tomorrow))
}

/// Открыть вчера. Ошибка, если дня нет.
pub fn open_past() -> Result<(Store, String)> {
    let store = load_store()?;
    let yesterday = date::yesterday();
    if !store.has_day(&yesterday) {
        bail!("вчерашний день не найден: {}", yesterday);
    }
    Ok((store, yesterday))
}

/// Открыть конкретную дату.
///
/// - Если дата в прошлом и дня нет — ошибка.
/// - Если дата сегодня/в будущем — создаёт, если нет.
pub fn open_date(date_str: &str) -> Result<(Store, String)> {
    date::validate(date_str)?;
    let mut store = load_store()?;

    if store.has_day(date_str) {
        return Ok((store, date_str.to_string()));
    }

    if date::is_past(date_str)? {
        bail!("день не найден: {}", date_str);
    }

    store.ensure_day(date_str);
    storage::save(&store)?;
    Ok((store, date_str.to_string()))
}

/// Удалить день. Ошибка, если дня нет.
pub fn delete_day(date_str: &str) -> Result<()> {
    date::validate(date_str)?;
    let mut store = load_store()?;

    if !store.remove_day(date_str) {
        bail!("день не найден: {}", date_str);
    }

    storage::save(&store)?;
    Ok(())
}

/// Добавить задачу в день. Создаёт день, если нет.
pub fn add_task(date_str: &str, task: Task) -> Result<()> {
    date::validate(date_str)?;
    let mut store = load_store()?;

    store.ensure_day(date_str);
    let day = store.get_day_mut(date_str).expect("день только что создан");

    day.add_task(task);
    storage::save(&store)?;
    Ok(())
}

/// Обновить задачу в дне.
pub fn update_task(date_str: &str, task: Task) -> Result<()> {
    date::validate(date_str)?;
    let mut store = load_store()?;

    let day = store
        .get_day_mut(date_str)
        .ok_or_else(|| anyhow::anyhow!("день не найден: {}", date_str))?;

    let existing = day
        .find_task_mut(&task.id)
        .ok_or_else(|| anyhow::anyhow!("задача не найдена: {}", task.id))?;

    *existing = task;
    day.sort_tasks();
    storage::save(&store)?;
    Ok(())
}

/// Удалить задачу.
pub fn delete_task(date_str: &str, task_id: &str) -> Result<()> {
    date::validate(date_str)?;
    let mut store = load_store()?;

    let day = store
        .get_day_mut(date_str)
        .ok_or_else(|| anyhow::anyhow!("день не найден: {}", date_str))?;

    if !day.remove_task(task_id) {
        bail!("задача не найдена: {}", task_id);
    }

    storage::save(&store)?;
    Ok(())
}

/// Установить комментарий задачи.
#[allow(dead_code)]
pub fn set_task_comment(date_str: &str, task_id: &str, comment: String) -> Result<()> {
    date::validate(date_str)?;
    let mut store = load_store()?;

    let day = store
        .get_day_mut(date_str)
        .ok_or_else(|| anyhow::anyhow!("день не найден: {}", date_str))?;

    let task = day
        .find_task_mut(task_id)
        .ok_or_else(|| anyhow::anyhow!("задача не найдена: {}", task_id))?;

    task.comment = comment;
    storage::save(&store)?;
    Ok(())
}

// --- Направления ---
/// Обновить направление по старому имени.
pub fn update_direction(old_name: &str, updated: crate::model::direction::Direction) -> Result<()> {
    let mut store = load_store()?;

    // Сначала проверяем, что направление существует.
    if !store.has_direction(old_name) {
        bail!("направление не найдено: {}", old_name);
    }

    // Проверяем уникальность нового имени (до взятия мутабельной ссылки).
    if old_name != updated.name && store.has_direction(&updated.name) {
        bail!("направление уже существует: {}", updated.name);
    }

    // Теперь безопасно берём мутабельную ссылку.
    let existing = store
        .find_direction_mut(old_name)
        .expect("направление только что проверено");

    *existing = updated;
    store.sort_directions();
    storage::save(&store)?;
    Ok(())
}

/// Добавить направление.
pub fn add_direction(direction: crate::model::direction::Direction) -> Result<()> {
    let mut store = load_store()?;
    if store.has_direction(&direction.name) {
        bail!("направление уже существует: {}", direction.name);
    }
    store.add_direction(direction);
    storage::save(&store)?;
    Ok(())
}

/// Удалить направление.
pub fn delete_direction(name: &str) -> Result<()> {
    let mut store = load_store()?;
    if !store.remove_direction(name) {
        bail!("направление не найдено: {}", name);
    }
    storage::save(&store)?;
    Ok(())
}

// --- Статистика ---

/// Данные для команды stats.
pub struct Stats {
    pub total_tasks: usize,
    pub done_tasks: usize,
    pub done_this_week: usize,
    pub by_month: Vec<(String, usize)>,
}

/// Собрать статистику.
pub fn collect_stats() -> Result<Stats> {
    let store = load_store()?;

    let mut total_tasks = 0usize;
    let mut done_tasks = 0usize;
    let mut done_this_week = 0usize;

    let today = chrono::Local::now().date_naive();
    let week_start = today - chrono::Duration::days(7);

    // Месяцы: ключ "мм-гггг" -> счётчик.
    let mut month_counts: std::collections::BTreeMap<String, usize> =
        std::collections::BTreeMap::new();

    for (date_str, day) in &store.days {
        let parsed = match date::parse(date_str) {
            Ok(d) => d,
            Err(_) => continue,
        };

        for task in &day.tasks {
            total_tasks += 1;
            if task.is_done() {
                done_tasks += 1;

                if parsed >= week_start && parsed <= today {
                    done_this_week += 1;
                }

                let month_key = parsed.format("%m-%Y").to_string();
                *month_counts.entry(month_key).or_insert(0) += 1;
            }
        }
    }

    // Преобразуем в вектор, сортируем по убыванию.
    let mut by_month: Vec<(String, usize)> = month_counts.into_iter().collect();
    by_month.sort_by(|a, b| b.0.cmp(&a.0));

    Ok(Stats {
        total_tasks,
        done_tasks,
        done_this_week,
        by_month,
    })
}

// --- Вспомогательное ---

/// Загрузить Store с проверкой версии и бэкапом.
fn load_store() -> Result<Store> {
    // Пробуем создать бэкап (если нужно) — но не падаем, если не получилось.
    let _ = storage::maybe_backup();

    let store = storage::load()?;
    storage::check_version(&store)?;
    Ok(store)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::direction::Direction;
    use crate::model::task::{Priority, Task};
    use std::sync::Mutex;

    fn setup() -> tempfile::TempDir {
        let dir = tempfile::tempdir().expect("tempdir");
        unsafe {
            std::env::set_var("DN_HOME", dir.path());
        }
        dir
    }

    fn cleanup() {
        unsafe {
            std::env::remove_var("DN_HOME");
        }
    }

    fn sample_task(title: &str) -> Task {
        Task::new(title.to_string())
    }

    // --- Дни ---

    #[test]
    fn test_open_today_creates() {
        let _guard = crate::model::TEST_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let _dir = setup();

        let (store, date) = open_today().unwrap();
        assert!(store.has_day(&date));

        cleanup();
    }

    #[test]
    fn test_open_today_idempotent() {
        let _guard = crate::model::TEST_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let _dir = setup();

        let (_, date1) = open_today().unwrap();
        let (_, date2) = open_today().unwrap();
        assert_eq!(date1, date2);

        cleanup();
    }

    #[test]
    fn test_open_next_creates() {
        let _guard = crate::model::TEST_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let _dir = setup();

        let (store, date) = open_next().unwrap();
        assert!(store.has_day(&date));
        assert_eq!(date, date::tomorrow());

        cleanup();
    }

    #[test]
    fn test_open_past_missing() {
        let _guard = crate::model::TEST_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let _dir = setup();

        // Вчерашнего дня нет — ошибка.
        let result = open_past();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("не найден"));

        cleanup();
    }

    #[test]
    fn test_open_past_existing() {
        let _guard = crate::model::TEST_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let _dir = setup();

        // Создаём вчера.
        let yesterday = date::yesterday();
        let mut store = Store::new();
        store.ensure_day(&yesterday);
        storage::save(&store).unwrap();

        let (store, date) = open_past().unwrap();
        assert_eq!(date, yesterday);
        assert!(store.has_day(&yesterday));

        cleanup();
    }

    #[test]
    fn test_open_date_future_creates() {
        let _guard = crate::model::TEST_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let _dir = setup();

        let future = "31-12-2099";
        let (store, date) = open_date(future).unwrap();
        assert_eq!(date, future);
        assert!(store.has_day(future));

        cleanup();
    }

    #[test]
    fn test_open_date_past_missing() {
        let _guard = crate::model::TEST_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let _dir = setup();

        // Прошлое, дня нет — ошибка.
        let past = "01-01-2020";
        let result = open_date(past);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("не найден"));

        cleanup();
    }

    #[test]
    fn test_open_date_past_existing() {
        let _guard = crate::model::TEST_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let _dir = setup();

        // Создаём день в прошлом.
        let past = "01-01-2020";
        let mut store = Store::new();
        store.ensure_day(past);
        storage::save(&store).unwrap();

        let (store, date) = open_date(past).unwrap();
        assert_eq!(date, past);
        assert!(store.has_day(past));

        cleanup();
    }

    #[test]
    fn test_open_date_invalid() {
        let _guard = crate::model::TEST_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let _dir = setup();

        assert!(open_date("not-a-date").is_err());
        assert!(open_date("").is_err());

        cleanup();
    }

    #[test]
    fn test_delete_day() {
        let _guard = crate::model::TEST_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let _dir = setup();

        let future = "31-12-2099";
        open_date(future).unwrap();

        delete_day(future).unwrap();

        let store = storage::load().unwrap();
        assert!(!store.has_day(future));

        cleanup();
    }

    #[test]
    fn test_delete_day_missing() {
        let _guard = crate::model::TEST_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let _dir = setup();

        let result = delete_day("31-12-2099");
        assert!(result.is_err());

        cleanup();
    }

    // --- Задачи ---

    #[test]
    fn test_add_task() {
        let _guard = crate::model::TEST_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let _dir = setup();

        let date = "31-12-2099";
        add_task(date, sample_task("Тест")).unwrap();

        let store = storage::load().unwrap();
        let day = store.get_day(date).unwrap();
        assert_eq!(day.tasks.len(), 1);
        assert_eq!(day.tasks[0].title, "Тест");

        cleanup();
    }

    #[test]
    fn test_add_task_creates_day() {
        let _guard = crate::model::TEST_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let _dir = setup();

        let date = "31-12-2099";
        add_task(date, sample_task("Тест")).unwrap();

        let store = storage::load().unwrap();
        assert!(store.has_day(date));

        cleanup();
    }

    #[test]
    fn test_update_task() {
        let _guard = crate::model::TEST_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let _dir = setup();

        let date = "31-12-2099";
        let task = sample_task("Оригинал");
        let id = task.id.clone();
        add_task(date, task).unwrap();

        let mut updated = Task::new("Изменено".to_string());
        updated.id = id.clone();
        updated.priority = Priority::High;
        update_task(date, updated).unwrap();

        let store = storage::load().unwrap();
        let day = store.get_day(date).unwrap();
        assert_eq!(day.tasks[0].title, "Изменено");
        assert_eq!(day.tasks[0].priority, Priority::High);

        cleanup();
    }

    #[test]
    fn test_update_task_missing_day() {
        let _guard = crate::model::TEST_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let _dir = setup();

        let task = sample_task("Тест");
        let result = update_task("31-12-2099", task);
        assert!(result.is_err());

        cleanup();
    }

    #[test]
    fn test_update_task_missing_task() {
        let _guard = crate::model::TEST_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let _dir = setup();

        let date = "31-12-2099";
        add_task(date, sample_task("A")).unwrap();

        let mut other = Task::new("Другая".to_string());
        other.id = "zzzzzzzz".to_string();

        let result = update_task(date, other);
        assert!(result.is_err());

        cleanup();
    }

    #[test]
    fn test_delete_task() {
        let _guard = crate::model::TEST_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let _dir = setup();

        let date = "31-12-2099";
        let task = sample_task("Удалить");
        let id = task.id.clone();
        add_task(date, task).unwrap();

        delete_task(date, &id).unwrap();

        let store = storage::load().unwrap();
        assert!(store.get_day(date).unwrap().tasks.is_empty());

        cleanup();
    }

    #[test]
    fn test_delete_task_missing() {
        let _guard = crate::model::TEST_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let _dir = setup();

        let result = delete_task("31-12-2099", "zzzzzzzz");
        assert!(result.is_err());

        cleanup();
    }

    #[test]
    fn test_set_task_comment() {
        let _guard = crate::model::TEST_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let _dir = setup();

        let date = "31-12-2099";
        let task = sample_task("Тест");
        let id = task.id.clone();
        add_task(date, task).unwrap();

        set_task_comment(date, &id, "Сделано".to_string()).unwrap();

        let store = storage::load().unwrap();
        let task = &store.get_day(date).unwrap().tasks[0];
        assert_eq!(task.comment, "Сделано");
        assert!(task.is_done());

        cleanup();
    }

    // --- Направления ---

    #[test]
    fn test_add_direction() {
        let _guard = crate::model::TEST_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let _dir = setup();

        let d = Direction::new("Здоровье".to_string());
        add_direction(d).unwrap();

        let store = storage::load().unwrap();
        assert!(store.has_direction("Здоровье"));

        cleanup();
    }

    #[test]
    fn test_add_direction_duplicate() {
        let _guard = crate::model::TEST_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let _dir = setup();

        let d1 = Direction::new("Здоровье".to_string());
        add_direction(d1).unwrap();

        let d2 = Direction::new("Здоровье".to_string());
        let result = add_direction(d2);
        assert!(result.is_err());

        cleanup();
    }

    #[test]
    fn test_update_direction() {
        let _guard = crate::model::TEST_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let _dir = setup();

        let d = Direction::new("Здоровье".to_string());
        add_direction(d).unwrap();

        let mut updated = Direction::new("Здоровье".to_string());
        updated.description = "Новое описание".to_string();
        updated.priority = Priority::High;

        update_direction("Здоровье", updated).unwrap();

        let store = storage::load().unwrap();
        let d = store.find_direction("Здоровье").unwrap();
        assert_eq!(d.description, "Новое описание");
        assert_eq!(d.priority, Priority::High);

        cleanup();
    }

    #[test]
    fn test_update_direction_rename() {
        let _guard = crate::model::TEST_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let _dir = setup();

        let d = Direction::new("Старое".to_string());
        add_direction(d).unwrap();

        let mut renamed = Direction::new("Новое".to_string());
        renamed.description = "Описание".to_string();

        update_direction("Старое", renamed).unwrap();

        let store = storage::load().unwrap();
        assert!(!store.has_direction("Старое"));
        assert!(store.has_direction("Новое"));

        cleanup();
    }

    #[test]
    fn test_update_direction_rename_conflict() {
        let _guard = crate::model::TEST_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let _dir = setup();

        add_direction(Direction::new("A".to_string())).unwrap();
        add_direction(Direction::new("B".to_string())).unwrap();

        // Переименовать A в B — конфликт.
        let mut renamed = Direction::new("B".to_string());
        renamed.description = "".to_string();
        let result = update_direction("A", renamed);
        assert!(result.is_err());

        cleanup();
    }

    #[test]
    fn test_update_direction_missing() {
        let _guard = crate::model::TEST_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let _dir = setup();

        let d = Direction::new("Нет".to_string());
        let result = update_direction("Нет", d);
        assert!(result.is_err());

        cleanup();
    }

    #[test]
    fn test_delete_direction() {
        let _guard = crate::model::TEST_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let _dir = setup();

        add_direction(Direction::new("Здоровье".to_string())).unwrap();
        delete_direction("Здоровье").unwrap();

        let store = storage::load().unwrap();
        assert!(!store.has_direction("Здоровье"));

        cleanup();
    }

    #[test]
    fn test_delete_direction_missing() {
        let _guard = crate::model::TEST_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let _dir = setup();

        let result = delete_direction("Нет");
        assert!(result.is_err());

        cleanup();
    }

    // --- Статистика ---

    #[test]
    fn test_collect_stats_empty() {
        let _guard = crate::model::TEST_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let _dir = setup();

        let stats = collect_stats().unwrap();
        assert_eq!(stats.total_tasks, 0);
        assert_eq!(stats.done_tasks, 0);
        assert_eq!(stats.done_this_week, 0);
        assert!(stats.by_month.is_empty());

        cleanup();
    }

    #[test]
    fn test_collect_stats_mixed() {
        let _guard = crate::model::TEST_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let _dir = setup();

        let date = "31-12-2099";

        // Две задачи: одна выполнена, одна нет.
        let t1 = sample_task("A");
        let id1 = t1.id.clone();
        add_task(date, t1).unwrap();

        let t2 = sample_task("B");
        add_task(date, t2).unwrap();

        set_task_comment(date, &id1, "готово".to_string()).unwrap();

        let stats = collect_stats().unwrap();
        assert_eq!(stats.total_tasks, 2);
        assert_eq!(stats.done_tasks, 1);

        cleanup();
    }
}
