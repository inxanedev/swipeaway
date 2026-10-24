use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

const MAX_RECENT: usize = 8;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub recent_folders: Vec<PathBuf>,
    #[serde(default)]
    pub last_root: Option<PathBuf>,
}

impl Config {
    pub fn load() -> Self {
        let Some(path) = Self::path() else {
            return Self::default();
        };
        let Ok(data) = std::fs::read_to_string(path) else {
            return Self::default();
        };
        serde_json::from_str(&data).unwrap_or_default()
    }

    pub fn save(&self) {
        let Some(path) = Self::path() else {
            return;
        };
        let Some(parent) = path.parent() else {
            return;
        };
        if std::fs::create_dir_all(parent).is_err() {
            return;
        }
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let _ = std::fs::write(path, json);
        }
    }

    pub fn remember(&mut self, folder: &Path) {
        self.last_root = Some(folder.to_path_buf());
        self.recent_folders.retain(|p| !same_path(p, folder));
        self.recent_folders.insert(0, folder.to_path_buf());
        self.recent_folders.truncate(MAX_RECENT);
        self.save();
    }

    pub fn forget(&mut self, folder: &Path) {
        self.recent_folders.retain(|p| !same_path(p, folder));
        if self
            .last_root
            .as_deref()
            .is_some_and(|p| same_path(p, folder))
        {
            self.last_root = self.recent_folders.first().cloned();
        }
        self.save();
    }

    fn path() -> Option<PathBuf> {
        dirs::config_dir().map(|dir| dir.join("SwipeAway").join("settings.json"))
    }
}

fn same_path(a: &Path, b: &Path) -> bool {
    crate::util::same_path(a, b)
}
