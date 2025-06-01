use std::cmp::Ordering;
use std::{
    fmt, fs, io,
    path::{Path, PathBuf},
};
#[derive(Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct DisplayablePathBuf(PathBuf);

impl DisplayablePathBuf {
    pub fn get(&self) -> &PathBuf {
        &self.0
    }
}

impl From<PathBuf> for DisplayablePathBuf {
    fn from(value: PathBuf) -> Self {
        Self(value)
    }
}

impl fmt::Display for DisplayablePathBuf {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.file_name().unwrap_or_default().display().fmt(f)
    }
}

pub fn get_entries<P: AsRef<Path>>(dirname: P) -> io::Result<Vec<PathBuf>> {
    let entries = fs::read_dir(dirname.as_ref())?
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .collect::<Vec<_>>();
    Ok(entries)
}

pub fn sort_entries(entries: &mut [PathBuf]) {
    entries.sort_by(|a, b| {
        let order = a
            .to_string_lossy()
            .to_lowercase()
            .trim_start_matches(".")
            .cmp(b.to_string_lossy().to_lowercase().trim_start_matches("."));

        if !(a.is_dir() ^ b.is_dir()) {
            order
        } else if a.is_dir() {
            Ordering::Less
        } else {
            Ordering::Greater
        }
    });
}

pub fn is_dotfile<P: AsRef<Path>>(entry: P) -> bool {
    entry
        .as_ref()
        .file_name()
        .is_some_and(|name| name.to_string_lossy().starts_with("."))
}
