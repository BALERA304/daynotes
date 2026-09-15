use anyhow::{Context, Result, bail};
use chrono::{Duration, Local, NaiveDate};

/// Формат даты, который используется в JSON и CLI.
pub const DATE_FORMAT: &str = "%d-%m-%Y";

/// Сегодняшняя дата в формате `дд-мм-гггг`.
pub fn today() -> String {
    Local::now().date_naive().format(DATE_FORMAT).to_string()
}

/// Завтрашняя дата.
pub fn tomorrow() -> String {
    (Local::now().date_naive() + Duration::days(1))
        .format(DATE_FORMAT)
        .to_string()
}

/// Вчерашняя дата.
pub fn yesterday() -> String {
    (Local::now().date_naive() - Duration::days(1))
        .format(DATE_FORMAT)
        .to_string()
}

/// Распарсить строку `дд-мм-гггг` в `NaiveDate`.
pub fn parse(s: &str) -> Result<NaiveDate> {
    NaiveDate::parse_from_str(s, DATE_FORMAT)
        .with_context(|| format!("неверный формат даты: '{}' (ожидается дд-мм-гггг)", s))
}

/// Отформатировать `NaiveDate` в `дд-мм-гггг`.
pub fn format(date: NaiveDate) -> String {
    date.format(DATE_FORMAT).to_string()
}

/// День недели на русском.
pub fn weekday_ru(date: NaiveDate) -> &'static str {
    use chrono::Datelike;
    match date.weekday() {
        chrono::Weekday::Mon => "понедельник",
        chrono::Weekday::Tue => "вторник",
        chrono::Weekday::Wed => "среда",
        chrono::Weekday::Thu => "четверг",
        chrono::Weekday::Fri => "пятница",
        chrono::Weekday::Sat => "суббота",
        chrono::Weekday::Sun => "воскресенье",
    }
}

/// Проверить, что дата в прошлом (строго раньше сегодня).
pub fn is_past(date: &str) -> Result<bool> {
    let parsed = parse(date)?;
    let today = Local::now().date_naive();
    Ok(parsed < today)
}

/// Проверить, что дата сегодня.
pub fn is_today(date: &str) -> Result<bool> {
    Ok(parse(date)? == Local::now().date_naive())
}

/// Проверить, что дата в будущем (строго позже сегодня).
pub fn is_future(date: &str) -> Result<bool> {
    let parsed = parse(date)?;
    let today = Local::now().date_naive();
    Ok(parsed > today)
}

/// Проверить, что дата сегодня или в будущем.
pub fn is_today_or_future(date: &str) -> Result<bool> {
    let parsed = parse(date)?;
    let today = Local::now().date_naive();
    Ok(parsed >= today)
}

/// Сгенерировать короткий id (8 символов).
pub fn generate_id() -> String {
    nanoid::nanoid!(8)
}

/// Проверка формата даты. Возвращает Ok, если валидно.
pub fn validate(s: &str) -> Result<()> {
    if s.is_empty() {
        bail!("дата не может быть пустой");
    }
    parse(s)?;
    Ok(())
}
