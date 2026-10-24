use std::path::{Path, PathBuf};

use crate::core::scan::{JUNK_DIR, MANUAL_DIR};
use crate::util::{path_key, path_within};

#[derive(Debug, Clone)]
pub struct SafetyError {
    pub title: &'static str,
    pub message: String,
}

impl std::fmt::Display for SafetyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.title, self.message)
    }
}

impl std::error::Error for SafetyError {}

pub fn validate_target(path: &Path) -> Result<PathBuf, SafetyError> {
    if path.as_os_str().is_empty() {
        return Err(SafetyError {
            title: "No folder selected",
            message: "Choose a folder to start cleaning.".to_owned(),
        });
    }

    if !path.exists() {
        return Err(SafetyError {
            title: "Folder not found",
            message: format!("`{}` no longer exists.", path.display()),
        });
    }

    if !path.is_dir() {
        return Err(SafetyError {
            title: "Not a folder",
            message: "SwipeAway works on folders. Pick a folder instead of a file.".to_owned(),
        });
    }

    let canonical = dunce::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
    let display = canonical.display().to_string();

    if is_drive_root(&canonical) || is_drive_root(path) {
        return Err(SafetyError {
            title: "That folder is too broad",
            message: format!(
                "`{display}` is the root of a drive. Point SwipeAway at a folder inside it instead \
                 — an entire drive is never a safe cleanup target."
            ),
        });
    }

    if let Some(component) = path
        .components()
        .filter_map(|c| c.as_os_str().to_str())
        .find(|name| name.eq_ignore_ascii_case(JUNK_DIR) || name.eq_ignore_ascii_case(MANUAL_DIR))
    {
        return Err(SafetyError {
            title: "That's a SwipeAway folder",
            message: format!(
                "`{component}` is one of SwipeAway's own holding folders. Choose the folder that \
                 contains it, or review its contents outside the app."
            ),
        });
    }

    if let Some(reason) = protected_reason(&canonical) {
        return Err(SafetyError {
            title: "Protected folder",
            message: format!("`{display}` {reason}"),
        });
    }

    Ok(canonical)
}

fn is_drive_root(path: &Path) -> bool {
    path.parent().is_none()
}

fn protected_reason(path: &Path) -> Option<&'static str> {
    for var in ["WINDIR", "SystemRoot"] {
        if let Some(root) = std::env::var_os(var).map(PathBuf::from)
            && path_within(path, &root)
        {
            return Some(
                "is inside the Windows system directory. These files keep your PC running and \
                     should never be cleaned with SwipeAway.",
            );
        }
    }

    for var in [
        "ProgramFiles",
        "ProgramFiles(x86)",
        "ProgramW6432",
        "ProgramData",
    ] {
        if let Some(root) = std::env::var_os(var).map(PathBuf::from)
            && path_within(path, &root)
        {
            return Some(
                "is a protected system location. Cleaning installed programs or shared \
                     application data can break software and Windows itself.",
            );
        }
    }

    if let Some(profile) = std::env::var_os("USERPROFILE").map(PathBuf::from)
        && path_key(path) == path_key(&profile)
    {
        return Some(
            "is your entire user profile. Choose a specific folder inside it (Downloads, \
                 Pictures, Documents, …) instead.",
        );
    }

    if let Some(public) = std::env::var_os("PUBLIC").map(PathBuf::from)
        && path_key(path) == path_key(&public)
    {
        return Some("is the shared Public profile. Choose a specific folder inside it instead.");
    }

    if let Some(system_drive) = std::env::var_os("SystemDrive").map(PathBuf::from) {
        let users = system_drive.join("Users");
        if path_key(path) == path_key(&users) {
            return Some(
                "contains every user profile on this PC. Choose a folder inside a specific profile instead.",
            );
        }
    }

    if is_direct_child_of_drive_root(path)
        && let Some(name) = path.file_name().and_then(|n| n.to_str())
    {
        let protected = [
            "$Recycle.Bin",
            "System Volume Information",
            "Recovery",
            "Windows.old",
            "Boot",
            "bootmgr",
            "pagefile.sys",
            "hiberfil.sys",
            "swapfile.sys",
            "PerfLogs",
            "$WinREAgent",
            "Config.Msi",
            "MSOCache",
        ];
        if protected
            .iter()
            .any(|candidate| name.eq_ignore_ascii_case(candidate))
        {
            return Some(
                "is a Windows recovery or system folder. It is not a safe cleanup target.",
            );
        }
    }

    None
}

fn is_direct_child_of_drive_root(path: &Path) -> bool {
    path.parent().is_some_and(is_drive_root)
}
