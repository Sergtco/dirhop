use std::fmt::Display;
use std::io;

use crossterm::{
    cursor, queue,
    style::{self, Stylize},
};

use crate::{
    Rect,
    tui::{Draw, Style},
};

pub struct Matcher<T: Display> {
    items: Vec<T>,
    entry_size: usize,
    bounds: Rect,
}

impl<T: Display> Matcher<T> {
    pub fn new(items: impl IntoIterator<Item = T>, bounds: Rect) -> Self {
        let items = Vec::from_iter(items);
        let entry_size = items
            .iter()
            .max_by_key(|x| x.to_string().len())
            .map(|x| x.to_string().len())
            .unwrap_or(1);

        Self {
            items,
            entry_size,
            bounds,
        }
    }

    pub fn get(&self, n: usize) -> Option<MatcherPage<T>> {
        let page_cap = self.bounds.width as usize / self.entry_size * self.bounds.height as usize;
        if self.items.is_empty() {
            return Some(MatcherPage::default());
        }
        Some(MatcherPage {
            items: self.items.chunks(page_cap).nth(n)?,
            ..Default::default()
        })
    }

    pub fn update_entries(&mut self, items: impl IntoIterator<Item = T>) {
        *self = Self::new(items, self.bounds);
    }
}

#[derive(Clone)]
pub struct MatcherPage<'a, T: Display> {
    items: &'a [T],
    input: String,
    style: Style<T>,
}

impl<'a, T: Display> Default for MatcherPage<'a, T> {
    fn default() -> Self {
        Self {
            items: &[],
            input: String::new(),
            style: |a| a.to_string().stylize(),
        }
    }
}

impl<'a, T: Display> MatcherPage<'a, T> {
    fn is_prefix_valid(&self, pfx: &str) -> bool {
        self.items
            .iter()
            .zip(Labeler::new())
            .find(|(_, label)| label.starts_with(pfx))
            .is_some()
    }

    fn find(&self, needle: &str) -> Option<&T> {
        self.items
            .iter()
            .zip(Labeler::new())
            .find_map(|(item, label)| needle.eq(&label).then_some(item))
    }

    pub fn iter(&self) -> impl Iterator<Item = (&T, String)> {
        self.items.iter().zip(Labeler::new())
    }

    pub fn item_size(&self) -> usize {
        self.items
            .iter()
            .map(|i| i.to_string().len())
            .max()
            .unwrap_or_default()
    }

    // Returns true if push is valid
    pub fn push_char(&mut self, c: char) -> bool {
        self.input.push(c);
        if !self.is_prefix_valid(&self.input) {
            self.input.pop();
            return false;
        }
        return true;
    }

    pub fn clear_input(&mut self) {
        self.input.clear();
    }

    pub fn result(&self) -> Option<&T> {
        self.find(&self.input)
    }

    pub fn set_style(&mut self, style: Style<T>) {
        self.style = style;
    }
}

impl<'a, T: std::fmt::Display> Draw for MatcherPage<'a, T> {
    fn draw<B>(&self, bounds: Rect, buf: &mut B) -> io::Result<()>
    where
        B: io::Write,
    {
        queue!(buf, cursor::MoveTo(bounds.x, bounds.y))?;
        let item_size = self.item_size() + 3;
        for (ind, bind) in self.iter().enumerate() {
            let row = ind % bounds.height as usize;
            let col = ind / bounds.height as usize;
            let (mut rest, mut typed) = (bind.1.as_str(), "");
            if let Some(right) = rest.strip_prefix(&self.input) {
                rest = right;
                typed = &self.input;
            }
            queue!(
                buf,
                cursor::MoveTo(
                    (bounds.x as usize + col * (item_size)) as u16,
                    bounds.y + row as u16
                ),
                style::Print(format!(
                    "[{}{}]{}",
                    typed.blue(),
                    rest,
                    (self.style)(bind.0)
                )),
            )?
        }
        Ok(())
    }
}

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
