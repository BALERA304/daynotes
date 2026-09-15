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
