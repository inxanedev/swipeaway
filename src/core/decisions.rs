use std::path::{Path, PathBuf};

pub fn move_entry(source: &Path, target_dir: &Path) -> Result<PathBuf, String> {
    let name = source
        .file_name()
        .map(|name| name.to_os_string())
        .ok_or_else(|| format!("`{}` has no file name.", source.display()))?;
    let is_dir = std::fs::symlink_metadata(source)
        .map(|metadata| metadata.is_dir())
        .unwrap_or(false);

    std::fs::create_dir_all(target_dir)
        .map_err(|err| format!("Could not create `{}`: {err}", target_dir.display()))?;

    let destination = unique_destination(target_dir, &name, is_dir);
    std::fs::rename(source, &destination).map_err(|err| {
        format!(
            "Could not move `{}`: {err}. It may be open in another program.",
            source.display()
        )
    })?;

    Ok(destination)
}

pub fn unique_destination(target_dir: &Path, name: &std::ffi::OsStr, is_dir: bool) -> PathBuf {
    let plain = target_dir.join(name);
    if !plain.exists() {
        return plain;
    }

    let name = name.to_string_lossy();
    let (stem, extension) = if is_dir {
        (name.to_string(), None)
    } else {
        match name.rsplit_once('.') {
            Some((stem, ext)) if !stem.is_empty() && !ext.is_empty() => {
                (stem.to_owned(), Some(ext.to_owned()))
            }
            _ => (name.to_string(), None),
        }
    };

    for index in 2..10_000 {
        let candidate = match &extension {
            Some(ext) => target_dir.join(format!("{stem} ({index}).{ext}")),
            None => target_dir.join(format!("{stem} ({index})")),
        };
        if !candidate.exists() {
            return candidate;
        }
    }

    target_dir.join(format!("{stem} (copy)"))
}
