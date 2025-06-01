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

pub fn get_entries<P: AsRef<Path>>(dirname: P) -> io::Result<impl Iterator<Item = PathBuf>> {
    let entries = fs::read_dir(&dirname)?.filter_map(|entry| entry.ok().map(|e| e.path()));
    Ok(entries)
}

pub fn sort_entries(entries: &mut [DisplayablePathBuf]) {
    entries.sort_by(|a, b| {
        let order = a
            .get()
            .to_string_lossy()
            .to_lowercase()
            .trim_start_matches(".")
            .cmp(
                b.get()
                    .to_string_lossy()
                    .to_lowercase()
                    .trim_start_matches("."),
            );

        if !(a.get().is_dir() ^ b.get().is_dir()) {
            order
        } else if a.get().is_dir() {
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
