use std::fs::Metadata;
use std::io;
use std::path::PathBuf;
use std::time::SystemTime;

#[cfg(windows)]
const FILE_ATTRIBUTE_HIDDEN: u32 = 0x0000_0002;
#[cfg(windows)]
const FILE_ATTRIBUTE_SYSTEM: u32 = 0x0000_0004;

#[derive(Debug, Clone)]
pub struct Entry {
    pub path: PathBuf,
    pub name: String,
    pub is_dir: bool,
    pub is_symlink: bool,
    pub size: u64,
    pub created: Option<SystemTime>,
    pub modified: Option<SystemTime>,
    pub accessed: Option<SystemTime>,
    pub hidden: bool,
    pub system: bool,
    pub readonly: bool,
}

impl Entry {
    pub fn load(path: PathBuf) -> io::Result<Self> {
        let metadata = std::fs::symlink_metadata(&path)?;
        Ok(Self::from_metadata(path, &metadata))
    }

    pub fn from_metadata(path: PathBuf, metadata: &Metadata) -> Self {
        let name = path
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_else(|| path.display().to_string());

        let hidden = {
            let mut hidden = name.starts_with('.');
            #[cfg(windows)]
            {
                use std::os::windows::fs::MetadataExt;
                hidden |= metadata.file_attributes() & FILE_ATTRIBUTE_HIDDEN != 0;
            }
            hidden
        };

        let system = {
            #[cfg(windows)]
            {
                use std::os::windows::fs::MetadataExt;
                metadata.file_attributes() & FILE_ATTRIBUTE_SYSTEM != 0
            }
            #[cfg(not(windows))]
            {
                false
            }
        };

        Self {
            path,
            name,
            is_dir: metadata.is_dir(),
            is_symlink: metadata.file_type().is_symlink(),
            size: metadata.len(),
            created: metadata.created().ok(),
            modified: metadata.modified().ok(),
            accessed: metadata.accessed().ok(),
            hidden,
            system,
            readonly: metadata.permissions().readonly(),
        }
    }
}

impl PartialEq for Entry {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path
    }
}

impl Eq for Entry {}
