use std::{
    cmp::Ordering,
    env,
    error::Error,
    fs,
    io::{self},
    path::{Path, PathBuf},
    process::exit,
    result,
};

use args::{Opts, usage};
use crossterm::{
    event::{Event, KeyCode, KeyEvent, KeyModifiers},
    style::Stylize,
    terminal,
    tty::IsTty,
};

mod args;
mod tui;
mod util;
use tui::Renderer;
use util::{DisplayablePathBuf, Matcher, Rect};

type Result<T> = result::Result<T, Box<dyn Error>>;
#[derive(Debug)]
enum AppEvent {
    Quit,
    Accept,
    Key(char),
    None,
}

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

    let bounds = {
        let (width, height) = terminal::size()?;
        Rect {
            x: 0,
            y: 0,
            width,
            height,
        }
    };
    let mut renderer = Renderer::new_with_bounds(bounds)?;
    let mut parent = fs::canonicalize(opts.base_path).unwrap_or_default();
    'outer: loop {
        let entries = get_entries(&parent).unwrap_or_else(|err| {
            renderer.restore().unwrap();
            eprintln!("Error reading directory {err}");
            exit(1)
        });

        let matcher = Matcher::new(
            &entries,
            Rect {
                x: bounds.x,
                y: bounds.y + 2,
                width: bounds.width,
                height: bounds.height - 2,
            },
        );

        let style = |bind: &&DisplayablePathBuf| match bind.get().is_dir() {
            true => bind.to_string().blue(),
            false => bind.to_string().stylize(),
        };

        let mut input = String::new();
        let mut cmdbuf = String::new();
        let mut page_num = 0;
        let mut curr_page = matcher.get(0).unwrap();
        renderer.draw_list(&input, parent.to_string_lossy().yellow(), curr_page, style)?;

        loop {
            match get_event()? {
                AppEvent::Accept => break 'outer,
                AppEvent::Quit => {
                    renderer.restore()?;
                    exit(0)
                }
                AppEvent::Key(c) if c.is_alphabetic() => input.push(c),
                AppEvent::Key(c) => cmdbuf.push(c),
                _ => (),
            };

            let mut input_changed = false;
            if !curr_page.is_prefix_valid(&input) {
                input.pop();
            } else if input.len() == 1 {
                input_changed = true
            } else if input.len() == 2 {
                break;
            }

            let mut page_changed = false;
            match &mut cmdbuf {
                cmdbuf if cmdbuf.ends_with(">>") => {
                    if let Some(next) = matcher.get(page_num + 1) {
                        page_num += 1;
                        curr_page = next;
                        page_changed = true;
                    }
                }
                cmdbuf if cmdbuf.ends_with(">>") => {
                    if let Some(prev) = matcher.get(page_num.wrapping_sub(1)) {
                        page_num -= 1;
                        curr_page = prev;
                        page_changed = true;
                    }
                }
                cmdbuf if cmdbuf.ends_with("..") => {
                    parent = parent
                        .parent()
                        .map(Path::to_path_buf)
                        .unwrap_or(PathBuf::from("/"));
                    continue 'outer;
                }
                _ => (),
            }

            if page_changed {
                input.clear();
                cmdbuf.clear();
            }
            if page_changed || input_changed {
                renderer.draw_list(&input, parent.to_string_lossy().yellow(), curr_page, style)?;
            }
        }

        if let Some(entry) = curr_page.find(&input) {
            parent = entry.get().clone();
            if entry.get().is_file() {
                break 'outer;
            }
        } else {
            renderer.restore()?;
            eprintln!("Wrong label!");
            exit(1);
        }
    }
    renderer.restore()?;
    println!("{}", parent.display());
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

fn get_event() -> io::Result<AppEvent> {
    match crossterm::event::read()? {
        Event::Key(KeyEvent {
            code, modifiers, ..
        }) => match (code, modifiers) {
            (KeyCode::Char('c'), KeyModifiers::CONTROL) => Ok(AppEvent::Quit),
            (KeyCode::Enter, KeyModifiers::NONE) => Ok(AppEvent::Accept),
            (KeyCode::Char(key), KeyModifiers::NONE) => Ok(AppEvent::Key(key)),
            _ => Ok(AppEvent::None),
        },
        _ => Ok(AppEvent::None),
    }
}
