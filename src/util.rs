use std::path::Path;
use std::process::Command;
use std::time::{Duration, SystemTime};

use chrono::{DateTime, Local};

pub fn item_count(count: usize) -> String {
    format!("{count} item{}", if count == 1 { "" } else { "s" })
}

pub fn format_size(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    if bytes < 1024 {
        return format!("{bytes} B");
    }
    let mut value = bytes as f64;
    let mut unit = 0usize;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    if value >= 100.0 {
        format!("{value:.0} {}", UNITS[unit])
    } else if value >= 10.0 {
        format!("{value:.1} {}", UNITS[unit])
    } else {
        format!("{value:.2} {}", UNITS[unit])
    }
}

pub fn format_size_short(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    if bytes < 1024 {
        return format!("{bytes} B");
    }
    let mut value = bytes as f64;
    let mut unit = 0usize;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    if value >= 10.0 {
        format!("{value:.1} {}", UNITS[unit])
    } else {
        format!("{value:.2} {}", UNITS[unit])
    }
}

pub fn format_datetime(time: Option<SystemTime>) -> String {
    let Some(time) = time else {
        return "Unknown".to_owned();
    };
    let dt: DateTime<Local> = time.into();
    dt.format("%b %d, %Y  %H:%M").to_string()
}

pub fn format_age(time: Option<SystemTime>) -> String {
    let Some(time) = time else {
        return String::new();
    };
    let now = SystemTime::now();
    let age = match now.duration_since(time) {
        Ok(age) => age,
        Err(_) => return "in the future".to_owned(),
    };
    let seconds = age.as_secs();
    if seconds < 60 {
        "just now".to_owned()
    } else if seconds < 3_600 {
        format!("{} min ago", seconds / 60)
    } else if seconds < 86_400 {
        format!("{} hr ago", seconds / 3_600)
    } else if seconds < 86_400 * 30 {
        format!("{} days ago", seconds / 86_400)
    } else if seconds < 86_400 * 365 {
        format!("{} mo ago", seconds / (86_400 * 30))
    } else {
        format!("{} yr ago", seconds / (86_400 * 365))
    }
}

pub fn format_duration(duration: Duration) -> String {
    let total = duration.as_secs();
    let hours = total / 3_600;
    let minutes = (total % 3_600) / 60;
    let seconds = total % 60;
    if hours > 0 {
        format!("{hours}:{minutes:02}:{seconds:02}")
    } else {
        format!("{minutes}:{seconds:02}")
    }
}

pub fn format_seconds(seconds: f64) -> String {
    if !seconds.is_finite() || seconds < 0.0 {
        return "Unknown".to_owned();
    }
    format_duration(Duration::from_secs_f64(seconds))
}

pub fn truncate_middle(text: &str, max_chars: usize) -> String {
    let chars: Vec<char> = text.chars().collect();
    if chars.len() <= max_chars || max_chars < 6 {
        return text.to_owned();
    }
    let keep = max_chars - 1;
    let front = keep / 2;
    let back = keep - front;
    let start: String = chars.iter().take(front).collect();
    let end: String = chars.iter().skip(chars.len() - back).collect();
    format!("{start}…{end}")
}

pub fn path_key(path: &Path) -> String {
    path.to_string_lossy()
        .trim_end_matches(['\\', '/'])
        .to_lowercase()
}

pub fn path_within(path: &Path, root: &Path) -> bool {
    let path = path_key(path);
    let root = path_key(root);
    if path == root {
        return true;
    }
    let prefix = if root.ends_with('\\') {
        root
    } else {
        format!("{root}\\")
    };
    path.starts_with(&prefix)
}

pub fn same_path(a: &Path, b: &Path) -> bool {
    path_key(a) == path_key(b)
}

#[cfg(windows)]
pub fn command_hidden(program: impl AsRef<std::ffi::OsStr>) -> Command {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    let mut cmd = Command::new(program);
    cmd.creation_flags(CREATE_NO_WINDOW);
    cmd
}

#[cfg(not(windows))]
pub fn command_hidden(program: impl AsRef<std::ffi::OsStr>) -> Command {
    Command::new(program)
}

#[cfg(windows)]
pub fn show_in_explorer(path: &Path) -> std::io::Result<()> {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    let mut cmd = Command::new("explorer.exe");
    cmd.creation_flags(CREATE_NO_WINDOW);
    cmd.raw_arg(format!("/select,\"{}\"", path.display()));
    cmd.spawn().map(|_| ())
}

#[cfg(not(windows))]
pub fn show_in_explorer(path: &Path) -> std::io::Result<()> {
    let path = if path.is_dir() {
        path.to_path_buf()
    } else {
        path.parent().unwrap_or(path).to_path_buf()
    };
    open::that(path)
}

pub fn open_path(path: &Path) -> std::io::Result<()> {
    open::that(path)
}

pub fn open_folder(path: &Path) -> std::io::Result<()> {
    open::that_detached(path)
}

pub fn spawn_instance(folder: &Path) -> std::io::Result<()> {
    let exe = std::env::current_exe()?;
    Command::new(exe)
        .arg(folder)
        .current_dir(folder)
        .spawn()
        .map(|_| ())
}

pub fn short_path(path: &Path, max_chars: usize) -> String {
    truncate_middle(&path.to_string_lossy(), max_chars)
}

pub fn extension_upper(path: &Path) -> String {
    path.extension()
        .map(|ext| ext.to_string_lossy().to_uppercase())
        .filter(|ext| !ext.is_empty())
        .unwrap_or_else(|| "FILE".to_owned())
}
