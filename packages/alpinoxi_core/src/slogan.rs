//! Rotating taglines ("OneClient is best!") with no repetition until the list is
//! exhausted, so the launcher never flickers between the same two lines.

use serde::{Deserialize, Serialize};

/// A tagline plus where it came from, so the UI can credit the original.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Slogan {
    pub text: String,
    /// `true` for lines Alpinoxi wrote, `false` for the borrowed classic.
    pub original: bool,
}

impl Slogan {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            original: true,
        }
    }

    /// The line that started the whole project.
    pub fn classic() -> Self {
        Self {
            text: "OneClient is best!".to_owned(),
            original: false,
        }
    }
}

/// Deterministic shuffler: no `rand` dependency, no allocation per tick.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SloganPicker {
    order: Vec<usize>,
    cursor: usize,
    seed: u64,
}

impl SloganPicker {
    pub fn new(lines: &[&str], seed: u64) -> Self {
        let mut picker = Self {
            order: (0..lines.len()).collect(),
            cursor: 0,
            seed,
        };
        picker.shuffle();
        picker
    }

    fn next_random(&mut self) -> u64 {
        // xorshift64*, plenty for shuffling six strings.
        self.seed ^= self.seed << 13;
        self.seed ^= self.seed >> 7;
        self.seed ^= self.seed << 17;
        self.seed
    }

    fn shuffle(&mut self) {
        let len = self.order.len();
        for i in (1..len).rev() {
            let j = (self.next_random() % (i as u64 + 1)) as usize;
            self.order.swap(i, j);
        }
    }

    pub fn next<'a>(&mut self, lines: &[&'a str]) -> Option<&'a str> {
        if lines.is_empty() {
            return None;
        }
        if self.cursor >= self.order.len() {
            self.shuffle();
            self.cursor = 0;
        }
        let index = self.order[self.cursor] % lines.len();
        self.cursor += 1;
        lines.get(index).copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn visits_every_line_once_per_cycle() {
        let lines = ["a", "b", "c"];
        let mut picker = SloganPicker::new(&lines, 0x5eed);
        let mut seen: Vec<&str> = (0..3).filter_map(|_| picker.next(&lines)).collect();
        seen.sort_unstable();
        assert_eq!(seen, vec!["a", "b", "c"]);
    }

    #[test]
    fn handles_empty_input() {
        let mut picker = SloganPicker::new(&[], 1);
        assert_eq!(picker.next(&[]), None);
    }
}
