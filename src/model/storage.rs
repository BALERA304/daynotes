use std::fs;
use std::path::PathBuf;
use std::time::{Duration, SystemTime};

use anyhow::{Context, Result};
use dirs::home_dir;

use super::store::{CURRENT_VERSION, Store};

/// Имя папки в домашней директории.
const DIR_NAME: &str = ".dn";

/// Имя основного файла.
const FILE_NAME: &str = "day_notes.json";

/// Имя файла бэкапа.
const BACKUP_NAME: &str = "backup.json";

/// Интервал бэкапа: раз в неделю.
const BACKUP_INTERVAL: Duration = Duration::from_secs(7 * 24 * 60 * 60);

/// Путь к папке `~/.dn/`.
pub fn dir_path() -> Result<PathBuf> {
    // Если задана переменная DN_HOME — используем её.
    if let Ok(custom) = std::env::var("DN_HOME") {
        if !custom.is_empty() {
            return Ok(PathBuf::from(custom));
        }
    }

    let home = home_dir().context("не удалось определить домашнюю директорию")?;
    Ok(home.join(DIR_NAME))
}

/// Путь к основному файлу `~/.dn/day_notes.json`.
pub fn file_path() -> Result<PathBuf> {
    Ok(dir_path()?.join(FILE_NAME))
}

/// Путь к бэкапу `~/.dn/backup.json`.
pub fn backup_path() -> Result<PathBuf> {
    Ok(dir_path()?.join(BACKUP_NAME))
}

/// Создать папку `~/.dn/` с правами 700, если её нет.
fn ensure_dir() -> Result<()> {
    let dir = dir_path()?;
    if !dir.exists() {
        fs::create_dir_all(&dir).with_context(|| format!("не удалось создать папку {:?}", dir))?;
        set_permissions_dir(&dir)?;
    }
    Ok(())
}

/// Установить права 700 на папку (только Unix).
#[cfg(unix)]
fn set_permissions_dir(path: &PathBuf) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let perms = fs::Permissions::from_mode(0o700);
    fs::set_permissions(path, perms)
        .with_context(|| format!("не удалось установить права на {:?}", path))?;
    Ok(())
}

#[cfg(not(unix))]
fn set_permissions_dir(_path: &PathBuf) -> Result<()> {
    Ok(())
}

/// Установить права 600 на файл (только Unix).
#[cfg(unix)]
fn set_permissions_file(path: &PathBuf) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let perms = fs::Permissions::from_mode(0o600);
    fs::set_permissions(path, perms)
        .with_context(|| format!("не удалось установить права на {:?}", path))?;
    Ok(())
}

#[cfg(not(unix))]
fn set_permissions_file(_path: &PathBuf) -> Result<()> {
    Ok(())
}

/// Прочитать Store из файла.
///
/// Если файла нет — создаёт новый пустой.
/// Если файл битый — возвращает ошибку (восстановление делает caller).
pub fn load() -> Result<Store> {
    ensure_dir()?;
    let path = file_path()?;

    if !path.exists() {
        let store = Store::new();
        save(&store)?;
        return Ok(store);
    }

    let content =
        fs::read_to_string(&path).with_context(|| format!("не удалось прочитать {:?}", path))?;

    let store: Store = serde_json::from_str(&content)
        .with_context(|| format!("не удалось распарсить {:?}", path))?;

    Ok(store)
}

/// Записать Store в файл.
pub fn save(store: &Store) -> Result<()> {
    ensure_dir()?;
    let path = file_path()?;

    let content =
        serde_json::to_string_pretty(store).context("не удалось сериализовать Store в JSON")?;

    fs::write(&path, content).with_context(|| format!("не удалось записать {:?}", path))?;

    set_permissions_file(&path)?;

    Ok(())
}

/// Проверить, битый ли основной файл.
///
/// Возвращает Ok(true), если файл есть и он битый.
/// Возвращает Ok(false), если файла нет или он валиден.
pub fn is_corrupted() -> Result<bool> {
    let path = file_path()?;
    if !path.exists() {
        return Ok(false);
    }
    let content =
        fs::read_to_string(&path).with_context(|| format!("не удалось прочитать {:?}", path))?;
    Ok(serde_json::from_str::<Store>(&content).is_err())
}

/// Создать новый пустой Store и записать его.
pub fn create_new() -> Result<Store> {
    let store = Store::new();
    save(&store)?;
    Ok(store)
}

/// Восстановить Store из бэкапа.
///
/// Копирует `backup.json` в `day_notes.json` и загружает его.
pub fn restore_from_backup() -> Result<Store> {
    let backup = backup_path()?;
    let file = file_path()?;

    if !backup.exists() {
        anyhow::bail!("бэкап не найден: {:?}", backup);
    }

    fs::copy(&backup, &file)
        .with_context(|| format!("не удалось скопировать {:?} в {:?}", backup, file))?;

    set_permissions_file(&file)?;

    load()
}

/// Создать бэкап, если он старше недели (или его нет).
///
/// Возвращает true, если бэкап был создан.
pub fn maybe_backup() -> Result<bool> {
    ensure_dir()?;
    let backup = backup_path()?;
    let file = file_path()?;

    // Если основного файла нет — бэкапить нечего.
    if !file.exists() {
        return Ok(false);
    }

    // Проверяем, нужен ли бэкап.
    let need_backup = if backup.exists() {
        let modified = fs::metadata(&backup)
            .with_context(|| format!("не удалось получить метаданные {:?}", backup))?
            .modified()
            .with_context(|| format!("не удалось получить время модификации {:?}", backup))?;
        let elapsed = SystemTime::now()
            .duration_since(modified)
            .unwrap_or(Duration::ZERO);
        elapsed >= BACKUP_INTERVAL
    } else {
        true
    };

    if !need_backup {
        return Ok(false);
    }

    fs::copy(&file, &backup).with_context(|| format!("не удалось создать бэкап {:?}", backup))?;

    set_permissions_file(&backup)?;

    Ok(true)
}

/// Проверить версию Store. Если версия не поддерживается — ошибка.
pub fn check_version(store: &Store) -> Result<()> {
    if store.version != CURRENT_VERSION {
        anyhow::bail!(
            "неподдерживаемая версия формата: {} (ожидается {})",
            store.version,
            CURRENT_VERSION
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::task::Task;
    use std::sync::Mutex;

    fn setup_temp_dir() -> tempfile::TempDir {
        let dir = tempfile::tempdir().expect("не удалось создать tempdir");
        // SAFETY: тесты защищены мьютексом TEST_LOCK,
        // поэтому гонки за переменную окружения нет.
        unsafe {
            std::env::set_var("DN_HOME", dir.path());
        }
        dir
    }

    fn cleanup() {
        // SAFETY: см. комментарий в setup_temp_dir.
        unsafe {
            std::env::remove_var("DN_HOME");
        }
    }

    #[test]
    fn test_dir_path_default() {
        let _guard = crate::model::TEST_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        cleanup();

        let path = dir_path().unwrap();
        assert!(path.ends_with(".dn"));
    }

    #[test]
    fn test_dir_path_custom() {
        let _guard = crate::model::TEST_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let dir = setup_temp_dir();

        let path = dir_path().unwrap();
        assert_eq!(path, dir.path());

        cleanup();
    }

    #[test]
    fn test_file_path() {
        let _guard = crate::model::TEST_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let dir = setup_temp_dir();

        let path = file_path().unwrap();
        assert_eq!(path, dir.path().join("day_notes.json"));

        cleanup();
    }

    #[test]
    fn test_backup_path() {
        let _guard = crate::model::TEST_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let dir = setup_temp_dir();

        let path = backup_path().unwrap();
        assert_eq!(path, dir.path().join("backup.json"));

        cleanup();
    }

    #[test]
    fn test_load_creates_file() {
        let _guard = crate::model::TEST_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let _dir = setup_temp_dir();

        // Файла нет — load() должен создать.
        let store = load().unwrap();
        assert!(file_path().unwrap().exists());
        assert!(store.days.is_empty());

        cleanup();
    }

    #[test]
    fn test_save_and_load_roundtrip() {
        let _guard = crate::model::TEST_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let _dir = setup_temp_dir();

        let mut store = Store::new();
        store.ensure_day("14-09-2026");
        store
            .get_day_mut("14-09-2026")
            .unwrap()
            .add_task(Task::new("Тест".to_string()));

        save(&store).unwrap();

        let loaded = load().unwrap();
        assert!(loaded.has_day("14-09-2026"));
        assert_eq!(loaded.get_day("14-09-2026").unwrap().tasks.len(), 1);

        cleanup();
    }

    #[test]
    fn test_is_corrupted_valid() {
        let _guard = crate::model::TEST_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let _dir = setup_temp_dir();

        // Создаём валидный файл.
        let store = Store::new();
        save(&store).unwrap();

        assert!(!is_corrupted().unwrap());

        cleanup();
    }

    #[test]
    fn test_is_corrupted_invalid() {
        let _guard = crate::model::TEST_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let _dir = setup_temp_dir();

        // Записываем мусор.
        let path = file_path().unwrap();
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, "garbage{").unwrap();

        assert!(is_corrupted().unwrap());

        cleanup();
    }

    #[test]
    fn test_is_corrupted_no_file() {
        let _guard = crate::model::TEST_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let _dir = setup_temp_dir();

        // Файла нет — не битый.
        assert!(!is_corrupted().unwrap());

        cleanup();
    }

    #[test]
    fn test_create_new() {
        let _guard = crate::model::TEST_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let _dir = setup_temp_dir();

        let store = create_new().unwrap();
        assert_eq!(store.version, crate::model::store::CURRENT_VERSION);
        assert!(file_path().unwrap().exists());

        cleanup();
    }

    #[test]
    fn test_restore_from_backup() {
        let _guard = crate::model::TEST_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let _dir = setup_temp_dir();

        // Создаём store и сохраняем как основной файл.
        let mut store = Store::new();
        store.ensure_day("14-09-2026");
        save(&store).unwrap();

        // Копируем в бэкап.
        let file = file_path().unwrap();
        let backup = backup_path().unwrap();
        std::fs::copy(&file, &backup).unwrap();

        // Ломаем основной файл.
        std::fs::write(&file, "garbage").unwrap();
        assert!(is_corrupted().unwrap());

        // Восстанавливаем.
        let restored = restore_from_backup().unwrap();
        assert!(restored.has_day("14-09-2026"));
        assert!(!is_corrupted().unwrap());

        cleanup();
    }

    #[test]
    fn test_restore_from_backup_missing() {
        let _guard = crate::model::TEST_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let _dir = setup_temp_dir();

        // Бэкапа нет — ошибка.
        let result = restore_from_backup();
        assert!(result.is_err());

        cleanup();
    }

    #[test]
    fn test_check_version_ok() {
        let store = Store::new();
        assert!(check_version(&store).is_ok());
    }

    #[test]
    fn test_check_version_wrong() {
        let mut store = Store::new();
        store.version = 999;
        assert!(check_version(&store).is_err());
    }
    #[test]
    fn test_maybe_backup_no_file() {
        let _guard = crate::model::TEST_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let _dir = setup_temp_dir();

        // Основного файла нет — бэкапить нечего.
        let created = maybe_backup().unwrap();
        assert!(!created);

        cleanup();
    }

    #[test]
    fn test_maybe_backup_creates() {
        let _guard = crate::model::TEST_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let _dir = setup_temp_dir();

        // Создаём основной файл.
        let store = Store::new();
        save(&store).unwrap();

        // Бэкапа нет — должен создаться.
        let created = maybe_backup().unwrap();
        assert!(created);
        assert!(backup_path().unwrap().exists());

        cleanup();
    }

    #[test]
    fn test_maybe_backup_skips_fresh() {
        let _guard = crate::model::TEST_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let _dir = setup_temp_dir();

        let store = Store::new();
        save(&store).unwrap();

        // Первый вызов — создаёт бэкап.
        assert!(maybe_backup().unwrap());

        // Второй вызов — бэкап свежий, не создаёт.
        let created = maybe_backup().unwrap();
        assert!(!created);

        cleanup();
    }
}
