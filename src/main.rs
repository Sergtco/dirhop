use std::{
    collections::BTreeMap,
    env,
    fs::{self, DirEntry},
    io::{self},
    process::exit,
};

use crossterm::{
    cursor,
    event::{Event, KeyModifiers},
    tty::IsTty,
};

mod tui;
use tui::{Rect, Renderer};

struct Labeler(u16);

impl Labeler {
    const TAG_INDEX_LIMIT: u16 = 675;

    fn new() -> Self {
        Self(0)
    }
}

impl Iterator for Labeler {
    type Item = String;

    fn next(&mut self) -> Option<String> {
        if self.0 > Self::TAG_INDEX_LIMIT {
            return None;
        }

        let first_letter = (self.0 / 26 + 97) as u8;
        let second_letter = (self.0 % 26 + 97) as u8;
        self.0 += 1;
        Some(String::from_utf8(vec![first_letter, second_letter]).expect("valid utf8 chars"))
    }
}
type Binds = BTreeMap<String, DirEntry>;

fn match_binds(ans: &str, binds: Binds) -> Binds {
    binds
        .into_iter()
        .filter(|(label, _)| label.starts_with(ans))
        .collect()
}

fn main() -> io::Result<()> {
    if !io::stderr().is_tty() {
        eprintln!("This is not a terminal, Babe!");
        exit(1);
    }

    let mut args = env::args();
    args.next();

    let path = args.next().unwrap_or(".".to_string());

    let entries = fs::read_dir(&path)?.filter_map(|entry| entry.ok());
    let labels = Labeler::new().into_iter();

    let mut binds = labels.zip(entries.into_iter()).collect::<Binds>();

    let mut renderer = {
        let (x, y) = cursor::position()?;
        Renderer::with_bounds(Rect {
            x,
            y,
            width: 10,
            height: 10,
        })?
    };

    let mut ans = String::new();
    renderer.draw_list(&ans, &binds)?;

    while ans.len() < 2 {
        match crossterm::event::read()? {
            Event::Key(event) => {
                if event.code.is_char('c') && event.modifiers == KeyModifiers::CONTROL {
                    renderer.restore()?;
                }
                if let Some(key) = event.code.as_char() {
                    if event.modifiers.is_empty() {
                        ans.push(key);
                        binds = match_binds(&ans, binds);

                        renderer.draw_list(&ans, &binds)?;
                    }
                }
            }
            _ => {}
        }
    }

    renderer.restore()?;

    if let Some(entry) = binds.get(&ans) {
        println!("{}", entry.path().to_string_lossy());
    } else {
        println!("Wrong label!");
    }
    Ok(())
}
