use std::path::{Path, PathBuf};
use std::time::Instant;

use super::decisions::move_entry;
use super::entry::Entry;
use super::scan::{JUNK_DIR, MANUAL_DIR, ScanReport};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Decision {
    Keep,
    Junk,
    Manual,
}

impl Decision {
    fn destination(self, root: &Path) -> Option<PathBuf> {
        match self {
            Self::Keep => None,
            Self::Junk => Some(root.join(JUNK_DIR)),
            Self::Manual => Some(root.join(MANUAL_DIR)),
        }
    }
}

#[derive(Debug, Clone)]
pub struct DecisionRecord {
    pub entry: Entry,
    pub decision: Decision,
    pub moved_to: Option<PathBuf>,
}

#[derive(Debug, Clone)]
pub struct DecisionOutcome {
    pub entry: Entry,
}

#[derive(Debug, Clone)]
pub struct UndoOutcome {
    pub entry: Entry,
    pub decision: Decision,
}

pub struct Session {
    pub root: PathBuf,
    pub entries: Vec<Entry>,
    pub index: usize,
    pub kept: u64,
    pub deleted: u64,
    pub manual: u64,
    pub deleted_bytes: u64,
    pub manual_bytes: u64,
    pub history: Vec<DecisionRecord>,
    pub warnings: Vec<String>,
    pub started_at: Instant,
}

impl Session {
    pub fn new(root: PathBuf, report: ScanReport) -> Self {
        Self {
            root,
            entries: report.entries,
            index: 0,
            kept: 0,
            deleted: 0,
            manual: 0,
            deleted_bytes: 0,
            manual_bytes: 0,
            history: Vec::new(),
            warnings: report.warnings,
            started_at: Instant::now(),
        }
    }

    pub fn current(&self) -> Option<&Entry> {
        self.entries.get(self.index)
    }

    pub fn total(&self) -> usize {
        self.entries.len()
    }

    pub fn is_finished(&self) -> bool {
        self.index >= self.entries.len()
    }

    pub fn junk_dir(&self) -> PathBuf {
        self.root.join(JUNK_DIR)
    }

    pub fn manual_dir(&self) -> PathBuf {
        self.root.join(MANUAL_DIR)
    }

    pub fn decide(&mut self, decision: Decision) -> Result<DecisionOutcome, String> {
        let entry = self
            .current()
            .cloned()
            .ok_or_else(|| "There are no more files to decide on.".to_owned())?;

        let destination = if let Some(target_dir) = decision.destination(&self.root) {
            Some(move_entry(&entry.path, &target_dir)?)
        } else {
            None
        };

        let bytes = if entry.is_dir { 0 } else { entry.size };
        match decision {
            Decision::Keep => self.kept += 1,
            Decision::Junk => {
                self.deleted += 1;
                self.deleted_bytes = self.deleted_bytes.saturating_add(bytes);
            }
            Decision::Manual => {
                self.manual += 1;
                self.manual_bytes = self.manual_bytes.saturating_add(bytes);
            }
        }

        self.history.push(DecisionRecord {
            entry: entry.clone(),
            decision,
            moved_to: destination.clone(),
        });
        self.index += 1;

        Ok(DecisionOutcome { entry })
    }

    pub fn can_undo(&self) -> bool {
        !self.history.is_empty()
    }

    pub fn undo_last(&mut self) -> Result<UndoOutcome, String> {
        let record = self
            .history
            .last()
            .cloned()
            .ok_or_else(|| "There is nothing to undo.".to_owned())?;

        if let Some(moved_to) = &record.moved_to {
            if !moved_to.exists() {
                return Err(format!(
                    "`{}` is no longer in the holding folder, so it can't be restored.",
                    record.entry.name
                ));
            }
            if record.entry.path.exists() {
                return Err(format!(
                    "Something with the name `{}` already exists at the original location.",
                    record.entry.name
                ));
            }
            if let Some(parent) = record.entry.path.parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|err| format!("Could not restore `{}`: {err}", record.entry.name))?;
            }
            std::fs::rename(moved_to, &record.entry.path).map_err(|err| {
                format!(
                    "Could not restore `{}`: {err}. It may be open in another program.",
                    record.entry.name
                )
            })?;
        }

        self.history.pop();
        self.index = self.index.saturating_sub(1);

        let bytes = if record.entry.is_dir {
            0
        } else {
            record.entry.size
        };
        match record.decision {
            Decision::Keep => self.kept = self.kept.saturating_sub(1),
            Decision::Junk => {
                self.deleted = self.deleted.saturating_sub(1);
                self.deleted_bytes = self.deleted_bytes.saturating_sub(bytes);
            }
            Decision::Manual => {
                self.manual = self.manual.saturating_sub(1);
                self.manual_bytes = self.manual_bytes.saturating_sub(bytes);
            }
        }

        Ok(UndoOutcome {
            entry: record.entry,
            decision: record.decision,
        })
    }

    pub fn entries_for(&self, decision: Decision) -> impl Iterator<Item = &Entry> {
        self.history
            .iter()
            .filter(move |record| record.decision == decision)
            .map(|record| &record.entry)
    }
}
