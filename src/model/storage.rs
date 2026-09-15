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
#[allow(dead_code)]
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
#[allow(dead_code)]

pub fn create_new() -> Result<Store> {
    let store = Store::new();
    save(&store)?;
    Ok(store)
}

/// Восстановить Store из бэкапа.
///
/// Копирует `backup.json` в `day_notes.json` и загружает его.
#[allow(dead_code)]
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
