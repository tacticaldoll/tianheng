//! One evaluation's reading of its source files: each path is read the first time a reader asks for it, and every
//! later request answers from that one reading, whether it succeeded or failed.
//!
//! A path is the key exactly as it was opened, never canonicalized: a file reached through a symlink resolves its
//! relative `#[path]` attributes from the directory it was opened in, so two spellings of one file are two sources
//! to the walk, and each is read as the walk names it. This is the one place 圭表 reads a source file's text.

use std::cell::RefCell;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::rc::Rc;

/// The texts one evaluation has read, by the path each was opened at.
#[derive(Default)]
pub(crate) struct SourceTexts {
    /// Each path's one reading: its text, or the read error's message, which each reader wraps in its own
    /// diagnostic exactly as it would wrap the error itself.
    read: RefCell<HashMap<PathBuf, Result<Rc<str>, String>>>,
    /// How many times each path has been read from the file system, incremented where the read happens and never
    /// where a request is answered from a reading already kept — the work the read-once requirement names, per path,
    /// so a path read twice beside one never read cannot hide behind a total.
    #[cfg(test)]
    reads: RefCell<HashMap<PathBuf, usize>>,
}

impl SourceTexts {
    /// The text of the file at `path`, reading it the first time any reader asks. A failed read is kept as its
    /// message and answered again to every later request, so every reader of one path meets one outcome.
    pub(crate) fn text(&self, path: &Path) -> Result<Rc<str>, String> {
        if let Some(reading) = self.read.borrow().get(path) {
            return reading.clone();
        }
        #[cfg(test)]
        {
            *self
                .reads
                .borrow_mut()
                .entry(path.to_path_buf())
                .or_default() += 1;
        }
        let reading = std::fs::read_to_string(path)
            .map(Rc::from)
            .map_err(|err| err.to_string());
        self.read
            .borrow_mut()
            .insert(path.to_path_buf(), reading.clone());
        reading
    }

    /// How many times each path has been read from the file system: one for every path any reader asked for, and
    /// no entry for a path nothing asked for.
    #[cfg(test)]
    pub(crate) fn reads(&self) -> HashMap<PathBuf, usize> {
        self.reads.borrow().clone()
    }
}
