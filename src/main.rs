use std::{
    cmp::Ordering,
    env,
    error::Error,
    fs::{self, canonicalize},
    io::{self},
    path::{Path, PathBuf},
    process::exit,
    result,
    str::FromStr,
};

use args::{Opts, usage};
use crossterm::{
    event::{Event, KeyCode, KeyEvent, KeyModifiers},
    style::Stylize,
    tty::IsTty,
};

mod args;
mod tui;
mod util;
use tui::Renderer;
use util::{DisplayablePathBuf, Matcher};

type Result<T> = result::Result<T, Box<dyn Error>>;

fn main() -> Result<()> {
    if !io::stderr().is_tty() {
        eprintln!("This is not a terminal!");
        exit(1);
    }

    let opts = Opts::from_args(env::args()).unwrap_or_else(|err| {
        eprintln!("Argument error: {err}.");
        usage();
        exit(1)
    });

    let mut path = opts.base_path.clone();
    let mut renderer = Renderer::new_fullscreen()?;
    'outer: loop {
        let entries = get_entries(&path).unwrap_or_else(|err| {
            renderer.restore().unwrap();
            eprintln!("Error reading directory {err}");
            exit(1)
        });

        let matcher = Matcher::new(&entries)?;

        let Some(entry) = entries.get(0) else {
            renderer.restore()?;
            exit(0)
        };

        let parent = fs::canonicalize(entry.get().parent().unwrap_or(&PathBuf::new()))
            .unwrap_or(PathBuf::default());

        let style = |bind: &&DisplayablePathBuf| match bind.get().is_dir() {
            true => bind.to_string().blue(),
            false => bind.to_string().stylize(),
        };

        let mut input = String::new();
        renderer.draw_list(
            &input,
            parent.to_string_lossy().yellow(),
            matcher.iter_all(),
            style,
        )?;

        let mut dot_pressed = false;
        while input.len() < 2 {
            match crossterm::event::read()? {
                Event::Key(KeyEvent {
                    code, modifiers, ..
                }) => {
                    if code.is_char('c') && modifiers == KeyModifiers::CONTROL {
                        renderer.restore()?;
                        break 'outer;
                    } else if code == KeyCode::Enter {
                        renderer.restore()?;
                        break 'outer;
                    } else if let Some(key) = code.as_char() {
                        if modifiers.is_empty() {
                            #[allow(unused_assignments)]
                            match key {
                                '.' if dot_pressed => {
                                    dot_pressed = false;
                                    path = canonicalize(path)
                                        .unwrap()
                                        .parent()
                                        .unwrap_or(&PathBuf::from_str("/").unwrap())
                                        .to_path_buf();
                                    continue 'outer;
                                }
                                '.' => {
                                    dot_pressed = true;
                                }
                                key => {
                                    input.push(key);

                                    if !matcher.is_valid_prefix(&input) {
                                        input.pop();
                                    }

                                    renderer.draw_list(
                                        &input,
                                        parent.to_string_lossy().yellow(),
                                        matcher.iter_all(),
                                        style,
                                    )?;
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }
        if let Some(entry) = matcher.find_exact(&input) {
            path = entry.get().clone();
        } else {
            renderer.restore()?;
            eprintln!("Wrong label!");
            exit(1);
        };
    }
    renderer.restore()?;
    println!("{}", path.display());
    Ok(())
}

fn get_entries<P: AsRef<Path>>(dirname: P) -> io::Result<Vec<DisplayablePathBuf>> {
    let mut entries = fs::read_dir(dirname.as_ref())?
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .map(|pb| DisplayablePathBuf::from(pb))
        .collect::<Vec<_>>();

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

    Ok(entries)
}
