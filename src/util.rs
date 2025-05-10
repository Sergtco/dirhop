use std::path::PathBuf;

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

pub type Binds = Vec<Bind>;
#[derive(Clone)]
pub struct Bind {
    pub label: String,
    pub path: PathBuf,
}

pub fn match_prefix(binds: &Binds, pfx: &str) -> Binds {
    binds
        .clone()
        .into_iter()
        .filter(|bind| bind.label.starts_with(pfx))
        .collect()
}
