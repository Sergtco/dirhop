use std::{
    collections::HashMap,
    env, fs, io,
    ops::Add,
    path::{Path, PathBuf},
};

use crossterm::{
    event::{Event, KeyCode, KeyEvent, KeyModifiers},
    style::Stylize,
    terminal,
};

use crate::{
    Rect,
    args::Opts,
    matcher::Matcher,
    tui::{Renderer, Style, Text},
    util::{DisplayablePathBuf, get_entries, is_dotfile, sort_entries},
};

pub struct App {
    renderer: Renderer,
    matcher: Matcher<DisplayablePathBuf>,
    opts: Opts,
    layout: HashMap<String, Rect>,
}

#[derive(Debug)]
enum InputEvent {
    Quit,
    Accept,
    Key(char),
    None,
    ToggleHidden,
    Clear,
}

#[derive(Debug)]
enum AppCommand {
    PageNext,
    PagePrev,
    Back,
}

impl App {
    pub fn new(opts: Opts) -> io::Result<Self> {
        let bounds = {
            let (width, height) = terminal::size()?;
            let (x, y) = (0, 0);
            Rect {
                x,
                y,
                width,
                height,
            }
        };
        let layout: HashMap<String, Rect> = [
            ("renderer".into(), bounds),
            (
                "text".into(),
                Rect {
                    x: 0,
                    y: 0,
                    width: bounds.width,
                    height: 2,
                },
            ),
            (
                "matcher".into(),
                Rect {
                    x: 0,
                    y: 2,
                    width: bounds.width,
                    height: bounds.height - 2,
                },
            ),
        ]
        .into();

        let entries = get_entries(&opts.path)?;
        let mut filtered = entries
            .filter(|entry| !is_dotfile(entry) || opts.show_hidden)
            .map(Into::into)
            .collect::<Vec<_>>();
        sort_entries(&mut filtered);

        let matcher = Matcher::new(filtered, layout["matcher"]);
        let renderer = Renderer::new_with_bounds(bounds)?;
        Ok(Self {
            renderer,
            matcher,
            opts,
            layout,
        })
    }

    pub fn run(&mut self) -> io::Result<String> {
        let mut parent = fs::canonicalize(&self.opts.path).unwrap_or_default();
        let style: Style<DisplayablePathBuf> = |a| match a.get().is_dir() {
            true => a.to_string().blue().bold(),
            false => a.to_string().stylize(),
        };

        'outer: loop {
            let mut input = String::new();

            let mut page_num = 0_usize;
            let mut curr_page = self.matcher.get(0).unwrap();
            curr_page.set_style(style);

            self.renderer
                .redraw(self.layout["text"], &Text::from(parent.display()))?;
            self.renderer.redraw(self.layout["matcher"], &curr_page)?;

            loop {
                // Input events
                let mut match_changed = false;
                match Self::read_input_event()? {
                    InputEvent::Accept => return Ok(parent.to_string_lossy().to_string()),
                    InputEvent::ToggleHidden => {
                        self.opts.show_hidden = !self.opts.show_hidden;
                        self.update_matcher(&parent)?;
                        continue 'outer;
                    }
                    InputEvent::Quit => {
                        return Ok(env::current_dir()?.display().to_string());
                    }
                    InputEvent::Key(c) => {
                        input.push(c);
                        if c.is_alphabetic() {
                            match_changed = curr_page.push_char(c);
                        }
                    }
                    InputEvent::Clear => {
                        curr_page.clear_input();
                    }
                    InputEvent::None => (),
                };

                // Check if matcher found result
                if let Some(entry) = curr_page.result() {
                    if entry.get().is_file() {
                        return Ok(entry.to_string());
                    }
                    parent = entry.get().clone();
                    self.update_matcher(&parent)?;
                    continue 'outer;
                }

                // Non matcher commands
                let mut next_page_num = page_num;
                match Self::parse_command(&input) {
                    Some(cmd) => match cmd {
                        AppCommand::PageNext => next_page_num = next_page_num.add(1),
                        AppCommand::PagePrev => next_page_num = next_page_num.wrapping_sub(1),
                        AppCommand::Back => {
                            parent = parent.parent().map(Path::to_path_buf).unwrap_or("/".into());
                            self.update_matcher(&parent)?;
                            continue 'outer;
                        }
                    },
                    None => (),
                }

                if next_page_num != page_num {
                    input.clear();
                    if let Some(next) = self.matcher.get(next_page_num) {
                        curr_page = next;
                        curr_page.set_style(style);
                        page_num = next_page_num;

                        self.renderer.redraw(self.layout["matcher"], &curr_page)?;
                    }
                }

                if match_changed {
                    self.renderer.redraw(self.layout["matcher"], &curr_page)?;
                }
            }
        }
    }

    fn update_matcher<P: AsRef<Path>>(&mut self, path: P) -> io::Result<()> {
        let new_entries = get_entries(&path)?;
        let mut filtered = new_entries
            .into_iter()
            .filter(|entry| !is_dotfile(entry) || self.opts.show_hidden)
            .map(PathBuf::into)
            .collect::<Vec<_>>();
        sort_entries(&mut filtered);

        self.matcher.update_entries(filtered);

        Ok(())
    }

    fn read_input_event() -> io::Result<InputEvent> {
        match crossterm::event::read()? {
            Event::Key(KeyEvent {
                code, modifiers, ..
            }) => match (code, modifiers) {
                (KeyCode::Char('c'), KeyModifiers::CONTROL) => Ok(InputEvent::Quit),
                (KeyCode::Char('h'), KeyModifiers::CONTROL) => Ok(InputEvent::ToggleHidden),
                (KeyCode::Enter, KeyModifiers::NONE) => Ok(InputEvent::Accept),
                (KeyCode::Esc, KeyModifiers::NONE) => Ok(InputEvent::Clear),
                (KeyCode::Char(key), KeyModifiers::NONE) => Ok(InputEvent::Key(key)),
                _ => Ok(InputEvent::None),
            },
            _ => Ok(InputEvent::None),
        }
    }

    fn parse_command(input: &str) -> Option<AppCommand> {
        match input.get(input.len().wrapping_sub(2)..) {
            Some(">>") => Some(AppCommand::PageNext),
            Some("<<") => Some(AppCommand::PagePrev),
            Some("..") => Some(AppCommand::Back),
            _ => None,
        }
    }

    pub fn restore(&mut self) -> io::Result<()> {
        self.renderer.restore()
    }
}
