use std::cmp::Ordering;
use std::path::{Path, PathBuf};

use super::entry::Entry;

pub const JUNK_DIR: &str = "swipeaway-junk";
pub const MANUAL_DIR: &str = "swipeaway-manual";

#[derive(Debug, Clone, Default)]
pub struct ScanReport {
    pub entries: Vec<Entry>,
    pub warnings: Vec<String>,
}

pub fn is_app_dir_name(name: &str) -> bool {
    name.eq_ignore_ascii_case(JUNK_DIR) || name.eq_ignore_ascii_case(MANUAL_DIR)
}

pub fn scan(root: &Path) -> std::io::Result<ScanReport> {
    let mut report = ScanReport::default();
    let mut entries = Vec::new();

    for item in std::fs::read_dir(root)? {
        let item = match item {
            Ok(item) => item,
            Err(err) => {
                report
                    .warnings
                    .push(format!("Could not read an entry: {err}"));
                continue;
            }
        };

        let name = item.file_name();
        if is_app_dir_name(&name.to_string_lossy()) {
            continue;
        }

        let path: PathBuf = item.path();
        match Entry::load(path.clone()) {
            Ok(entry) => entries.push(entry),
            Err(err) => report
                .warnings
                .push(format!("Could not inspect `{}`: {err}", path.display())),
        }
    }

    entries.sort_by(compare_entries);
    report.entries = entries;
    Ok(report)
}

fn compare_entries(a: &Entry, b: &Entry) -> Ordering {
    b.is_dir
        .cmp(&a.is_dir)
        .then_with(|| natural_cmp(&a.name, &b.name))
}

fn natural_cmp(a: &str, b: &str) -> Ordering {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    let mut ai = 0usize;
    let mut bi = 0usize;

    while ai < a_chars.len() && bi < b_chars.len() {
        if a_chars[ai].is_ascii_digit() && b_chars[bi].is_ascii_digit() {
            let a_start = ai;
            let b_start = bi;
            while ai < a_chars.len() && a_chars[ai].is_ascii_digit() {
                ai += 1;
            }
            while bi < b_chars.len() && b_chars[bi].is_ascii_digit() {
                bi += 1;
            }
            let a_num: String = a_chars[a_start..ai].iter().collect();
            let b_num: String = b_chars[b_start..bi].iter().collect();
            let a_trim = a_num.trim_start_matches('0');
            let b_trim = b_num.trim_start_matches('0');
            let a_trim = if a_trim.is_empty() { "0" } else { a_trim };
            let b_trim = if b_trim.is_empty() { "0" } else { b_trim };
            let order = a_trim
                .len()
                .cmp(&b_trim.len())
                .then_with(|| a_trim.cmp(b_trim))
                .then_with(|| a_num.len().cmp(&b_num.len()));
            if order != Ordering::Equal {
                return order;
            }
        } else {
            let order = a_chars[ai]
                .to_ascii_lowercase()
                .cmp(&b_chars[bi].to_ascii_lowercase());
            if order != Ordering::Equal {
                return order;
            }
            ai += 1;
            bi += 1;
        }
    }

    a_chars.len().cmp(&b_chars.len()).then_with(|| a.cmp(b))
}
