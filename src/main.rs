use std::{
    env,
    error::Error,
    fs, io,
    path::{Path, PathBuf},
    process::exit,
    result,
};

use crossterm::{
    event::{Event, KeyCode, KeyEvent, KeyModifiers},
    style::Stylize,
    terminal,
    tty::IsTty,
};

use dirhop::{
    args::{Opts, usage},
    get_entries,
    tui::Renderer,
    util::{DisplayablePathBuf, Matcher, Rect},
};

type Result<T> = result::Result<T, Box<dyn Error>>;
#[derive(Debug)]
enum AppEvent {
    Quit,
    Accept,
    Key(char),
    None,
    ToggleHidden,
    Clear,
}

fn main() -> Result<()> {
    if !io::stderr().is_tty() {
        eprintln!("This is not a terminal!");
        exit(1);
    }

    let mut opts = Opts::from_args(env::args()).unwrap_or_else(|err| {
        eprintln!("Argument error: {err}.");
        usage();
        exit(1)
    });

    let bounds = {
        let (width, height) = terminal::size()?;
        Rect {
            x: 2,
            y: 0,
            width: width - 2,
            height,
        }
    };
    let mut renderer = Renderer::new_with_bounds(bounds)?;
    let mut parent = fs::canonicalize(&opts.base_path).unwrap_or_default();
    'outer: loop {
        let entries = get_entries(&parent, &opts).unwrap_or_else(|err| {
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
        let mut pair = String::new();
        let mut page_num = 0_usize;
        let mut curr_page = matcher.get(0).unwrap();
        renderer.draw_list(&pair, parent.to_string_lossy().yellow(), curr_page, style)?;

        loop {
            match get_event()? {
                AppEvent::Accept => break 'outer,
                AppEvent::ToggleHidden => {
                    opts.show_hidden = !opts.show_hidden;
                    continue 'outer;
                }
                AppEvent::Quit => {
                    parent = env::current_dir()?;
                    break 'outer;
                }
                AppEvent::Key(c) => {
                    input.push(c);
                    if c.is_alphabetic() {
                        pair.push(c);
                    }
                }
                AppEvent::Clear => {
                    pair.clear();
                }
                AppEvent::None => (),
            };

            let mut pair_changed = false;
            if !curr_page.is_prefix_valid(&pair) {
                pair.pop();
            } else if pair.len() <= 1 {
                pair_changed = true
            } else if pair.len() == 2 {
                break;
            }

            let mut next_page_num = page_num;
            match input.get(input.len().wrapping_sub(2)..) {
                Some(">>") => {
                    next_page_num += 1;
                }
                Some("<<") => {
                    next_page_num = next_page_num.wrapping_sub(1);
                }
                Some("..") => {
                    parent = parent
                        .parent()
                        .map(Path::to_path_buf)
                        .unwrap_or(PathBuf::from("/"));
                    continue 'outer;
                }
                _ => (),
            }

            if next_page_num != page_num {
                input.clear();
                pair.clear();
                if let Some(next) = matcher.get(next_page_num) {
                    curr_page = next;
                    page_num = next_page_num;
                }
            }
            if next_page_num != page_num || pair_changed {
                renderer.draw_list(&pair, parent.to_string_lossy().yellow(), curr_page, style)?;
            }
        }

        if let Some(entry) = curr_page.find(&pair) {
            parent = entry.get().clone();
            if entry.get().is_file() {
                break 'outer;
            }
        }
    }
    renderer.restore()?;
    println!("{}", parent.display());
    Ok(())
}

fn get_event() -> io::Result<AppEvent> {
    match crossterm::event::read()? {
        Event::Key(KeyEvent {
            code, modifiers, ..
        }) => match (code, modifiers) {
            (KeyCode::Char('c'), KeyModifiers::CONTROL) => Ok(AppEvent::Quit),
            (KeyCode::Char('h'), KeyModifiers::CONTROL) => Ok(AppEvent::ToggleHidden),
            (KeyCode::Enter, KeyModifiers::NONE) => Ok(AppEvent::Accept),
            (KeyCode::Esc, KeyModifiers::NONE) => Ok(AppEvent::Clear),
            (KeyCode::Char(key), KeyModifiers::NONE) => Ok(AppEvent::Key(key)),
            _ => Ok(AppEvent::None),
        },
        _ => Ok(AppEvent::None),
    }
}
