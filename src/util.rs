use std::{
    fmt::{self, Display},
    path::PathBuf,
};

#[derive(Debug, Clone, Copy)]
pub struct Rect {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}

pub struct Labeler(u16);

impl Labeler {
    const TAG_INDEX_LIMIT: u16 = 675;

    pub fn new() -> Self {
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
            .unwrap_or_default();

        Self {
            items,
            entry_size,
            bounds,
        }
    }

    pub fn get(&self, n: usize) -> Option<MatcherPage<T>> {
        let page_cap = self.bounds.width as usize / self.entry_size * self.bounds.height as usize;
        Some(MatcherPage {
            items: self.items.chunks(page_cap).nth(n)?,
            entry_size: self.entry_size,
        })
    }
}

#[derive(Clone, Copy)]
pub struct MatcherPage<'a, T: Display> {
    items: &'a [T],
    entry_size: usize,
}

impl<'a, T: Display> MatcherPage<'a, T> {
    pub fn is_prefix_valid(&self, pfx: &str) -> bool {
        self.items
            .iter()
            .zip(Labeler::new())
            .find(|(_, label)| label.starts_with(pfx))
            .is_some()
    }

    pub fn find(&self, needle: &str) -> Option<&T> {
        self.items
            .iter()
            .zip(Labeler::new())
            .find_map(|(item, label)| needle.eq(&label).then_some(item))
    }

    pub fn iter(&self) -> impl Iterator<Item = (&T, String)> {
        self.items.iter().zip(Labeler::new())
    }
    pub fn item_size(&self) -> usize {
        self.entry_size
    }
}

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
        self.0
            .file_name()
            .map(|fname| fname.to_string_lossy())
            .unwrap_or("".into())
            .fmt(f)
    }
}
