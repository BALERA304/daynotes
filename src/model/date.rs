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
    // Проверяем, что длина ровно 10 символов: дд-мм-гггг.
    if s.len() != 10 {
        anyhow::bail!(
            "неверный формат даты: '{}' (ожидается дд-мм-гггг, 10 символов)",
            s
        );
    }

    // Проверяем, что разделители на месте.
    let bytes = s.as_bytes();
    if bytes[2] != b'-' || bytes[5] != b'-' {
        anyhow::bail!("неверный формат даты: '{}' (ожидается дд-мм-гггг)", s);
    }

    NaiveDate::parse_from_str(s, DATE_FORMAT).with_context(|| format!("неверная дата: '{}'", s))
}

/// Отформатировать `NaiveDate` в `дд-мм-гггг`.
#[allow(dead_code)]
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
#[allow(dead_code)]
pub fn is_today(date: &str) -> Result<bool> {
    Ok(parse(date)? == Local::now().date_naive())
}

/// Проверить, что дата в будущем (строго позже сегодня).
#[allow(dead_code)]
pub fn is_future(date: &str) -> Result<bool> {
    let parsed = parse(date)?;
    let today = Local::now().date_naive();
    Ok(parsed > today)
}

/// Проверить, что дата сегодня или в будущем.
#[allow(dead_code)]
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

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn test_parse_valid() {
        let d = parse("14-09-2026").unwrap();
        assert_eq!(d, NaiveDate::from_ymd_opt(2026, 9, 14).unwrap());
    }

    #[test]
    fn test_parse_invalid_format() {
        assert!(parse("2026-09-14").is_err()); // ISO не подходит
        assert!(parse("14/09/2026").is_err()); // слэши
        assert!(parse("14-09-26").is_err()); // короткий год
    }

    #[test]
    fn test_parse_invalid_date() {
        assert!(parse("32-09-2026").is_err()); // нет 32-го
        assert!(parse("14-13-2026").is_err()); // нет 13-го месяца
        assert!(parse("30-02-2026").is_err()); // февраль
    }

    #[test]
    fn test_parse_empty() {
        assert!(parse("").is_err());
    }

    #[test]
    fn test_format() {
        let d = NaiveDate::from_ymd_opt(2026, 9, 14).unwrap();
        assert_eq!(format(d), "14-09-2026");
    }

    #[test]
    fn test_format_parse_roundtrip() {
        let original = "14-09-2026";
        let parsed = parse(original).unwrap();
        let formatted = format(parsed);
        assert_eq!(formatted, original);
    }

    #[test]
    fn test_today_format() {
        let today = today();
        // Должно парситься без ошибок.
        assert!(parse(&today).is_ok());
        // Длина ровно 10: дд-мм-гггг.
        assert_eq!(today.len(), 10);
    }

    #[test]
    fn test_tomorrow_after_today() {
        let today = parse(&today()).unwrap();
        let tomorrow = parse(&tomorrow()).unwrap();
        assert_eq!(tomorrow - today, chrono::Duration::days(1));
    }

    #[test]
    fn test_yesterday_before_today() {
        let today = parse(&today()).unwrap();
        let yesterday = parse(&yesterday()).unwrap();
        assert_eq!(today - yesterday, chrono::Duration::days(1));
    }

    #[test]
    fn test_weekday_ru() {
        // 14-09-2026 — понедельник.
        let d = NaiveDate::from_ymd_opt(2026, 9, 14).unwrap();
        assert_eq!(weekday_ru(d), "понедельник");

        // 15-09-2026 — вторник.
        let d = NaiveDate::from_ymd_opt(2026, 9, 15).unwrap();
        assert_eq!(weekday_ru(d), "вторник");

        // 20-09-2026 — воскресенье.
        let d = NaiveDate::from_ymd_opt(2026, 9, 20).unwrap();
        assert_eq!(weekday_ru(d), "воскресенье");
    }

    #[test]
    fn test_is_past() {
        // Вчера — прошлое.
        assert!(is_past(&yesterday()).unwrap());
        // Сегодня — не прошлое.
        assert!(!is_past(&today()).unwrap());
        // Завтра — не прошлое.
        assert!(!is_past(&tomorrow()).unwrap());
    }

    #[test]
    fn test_is_today() {
        assert!(is_today(&today()).unwrap());
        assert!(!is_today(&yesterday()).unwrap());
        assert!(!is_today(&tomorrow()).unwrap());
    }

    #[test]
    fn test_is_future() {
        assert!(is_future(&tomorrow()).unwrap());
        assert!(!is_future(&today()).unwrap());
        assert!(!is_future(&yesterday()).unwrap());
    }

    #[test]
    fn test_is_today_or_future() {
        assert!(is_today_or_future(&today()).unwrap());
        assert!(is_today_or_future(&tomorrow()).unwrap());
        assert!(!is_today_or_future(&yesterday()).unwrap());
    }

    #[test]
    fn test_generate_id_length() {
        let id = generate_id();
        assert_eq!(id.len(), 8);
    }

    #[test]
    fn test_generate_id_unique() {
        let id1 = generate_id();
        let id2 = generate_id();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_generate_id_charset() {
        let id = generate_id();
        // nanoid использует A-Za-z0-9_-
        assert!(
            id.chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
        );
    }

    #[test]
    fn test_validate_ok() {
        assert!(validate("14-09-2026").is_ok());
    }

    #[test]
    fn test_validate_empty() {
        let result = validate("");
        assert!(result.is_err());
        let msg = result.unwrap_err().to_string();
        assert!(msg.contains("пустой"));
    }

    #[test]
    fn test_validate_invalid() {
        assert!(validate("не-дата").is_err());
    }
}
