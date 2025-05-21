use std::{
    error::Error,
    fmt::{self, Display},
    path::PathBuf,
};

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

pub struct Binds<T: Clone>(Vec<Bind<T>>);

impl<T: Clone> IntoIterator for Binds<T> {
    type Item = Bind<T>;
    type IntoIter = std::vec::IntoIter<Bind<T>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

#[derive(Clone)]
pub struct Bind<T: Clone> {
    pub label: String,
    pub item: T,
}

#[derive(Debug)]
pub struct TooMuchElems;

impl Display for TooMuchElems {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Too much elements")
    }
}

impl Error for TooMuchElems {}

impl<T: Clone> Binds<T> {
    pub fn new(items: impl IntoIterator<Item = T>) -> Result<Self, TooMuchElems> {
        let mut labeler = Labeler::new();
        Ok(Self(
            items
                .into_iter()
                .map(|item| {
                    Ok(Bind {
                        label: labeler.next().ok_or(TooMuchElems)?,
                        item,
                    })
                })
                .collect::<Result<Vec<_>, TooMuchElems>>()?,
        ))
    }

    pub fn is_valid_prefix(&self, pfx: &str) -> bool {
        self.0
            .iter()
            .filter(|bind| bind.label.starts_with(pfx))
            .count()
            > 0
    }

    pub fn iter(&self) -> impl Iterator<Item = &Bind<T>> {
        self.0.iter()
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
