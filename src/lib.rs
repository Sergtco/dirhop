use crate::util::DisplayablePathBuf;

use std::cmp::Ordering;
use std::path::Path;

use std::{fs, io};

pub mod args;
pub mod tui;
pub mod util;

use args::Opts;

pub fn get_entries<P: AsRef<Path>>(dirname: P, opts: &Opts) -> io::Result<Vec<DisplayablePathBuf>> {
    let mut entries = fs::read_dir(dirname.as_ref())?
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .filter(|entry| {
            !entry
                .file_name()
                .is_some_and(|name| name.to_string_lossy().starts_with("."))
                || opts.show_hidden
        })
        .map(|pb| DisplayablePathBuf::from(pb))
        .collect::<Vec<_>>();

    entries.sort_by(|a, b| {
        let order = a
            .to_string()
            .to_lowercase()
            .trim_start_matches(".")
            .cmp(b.to_string().to_lowercase().trim_start_matches("."));

        if !(a.get().is_dir() ^ b.get().is_dir()) {
            order
        } else if a.get().is_dir() {
            Ordering::Less
        } else {
            Ordering::Greater
        }
    });

    Ok(entries)
}
