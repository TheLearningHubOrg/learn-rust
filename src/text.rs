//! Stage 2: functions and types that borrow instead of copying.

use std::collections::BTreeMap;

use serde::Serialize;

/// Returns the first whitespace-separated word of `input`.
///
/// The result is a slice of `input`, so no bytes are copied. Elision ties the
/// output lifetime to the single input lifetime.
pub fn first_word(input: &str) -> &str {
    input.split_whitespace().next().unwrap_or("")
}

/// Returns the longer slice, or `left` on a tie.
///
/// Two input references means elision cannot pick one, so `'a` says the result
/// is valid only while both inputs are.
pub fn longest<'a>(left: &'a str, right: &'a str) -> &'a str {
    if right.len() > left.len() {
        right
    } else {
        left
    }
}

fn is_word_char(c: char) -> bool {
    c.is_alphanumeric() || c == '\''
}

/// An iterator over the words of a borrowed string.
///
/// The struct holds `&'a str`, so it cannot outlive the text it walks.
/// Each item is a sub-slice of that same text.
#[derive(Debug, Clone)]
pub struct Words<'a> {
    rest: &'a str,
}

impl<'a> Words<'a> {
    pub fn new(text: &'a str) -> Self {
        Self { rest: text }
    }
}

impl<'a> Iterator for Words<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        let start = self.rest.find(is_word_char)?;
        let tail = &self.rest[start..];
        let len = tail.find(|c: char| !is_word_char(c)).unwrap_or(tail.len());
        let (word, rest) = tail.split_at(len);
        self.rest = rest;
        Some(word)
    }
}

/// Counts words case-insensitively. Keys are owned because lowercasing allocates.
pub fn word_counts(text: &str) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    for word in Words::new(text) {
        *counts.entry(word.to_lowercase()).or_insert(0) += 1;
    }
    counts
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct WordCount {
    pub word: String,
    pub count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TextStats {
    pub words: usize,
    pub unique_words: usize,
    pub longest_word: String,
    pub top: Vec<WordCount>,
}

/// Summarises `text`. Only the values that leave the function are allocated.
pub fn stats(text: &str, top_n: usize) -> TextStats {
    let counts = word_counts(text);
    let words = counts.values().sum();
    let unique_words = counts.len();
    let longest_word = Words::new(text).fold("", longest).to_string();

    let mut top: Vec<WordCount> = counts
        .into_iter()
        .map(|(word, count)| WordCount { word, count })
        .collect();
    // Highest count first; ties sorted alphabetically so output is stable.
    top.sort_by(|a, b| b.count.cmp(&a.count).then_with(|| a.word.cmp(&b.word)));
    top.truncate(top_n);

    TextStats {
        words,
        unique_words,
        longest_word,
        top,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_word_borrows_from_input() {
        let line = String::from("  borrow checker rules");
        let word = first_word(&line);
        assert_eq!(word, "borrow");
        // The slice points into `line`'s heap buffer: no copy was made.
        assert!(line.as_ptr() <= word.as_ptr());
        assert_eq!(first_word("   "), "");
    }

    #[test]
    fn longest_prefers_left_on_tie() {
        assert_eq!(longest("abc", "xyz"), "abc");
        assert_eq!(longest("ab", "xyz"), "xyz");
    }

    #[test]
    fn words_skips_punctuation_and_keeps_apostrophes() {
        let words: Vec<&str> = Words::new("Don't panic! Rust, rust... RUST?").collect();
        assert_eq!(words, ["Don't", "panic", "Rust", "rust", "RUST"]);
    }

    #[test]
    fn words_handles_multibyte_text() {
        let words: Vec<&str> = Words::new("café naïve 🦀 crab").collect();
        assert_eq!(words, ["café", "naïve", "crab"]);
    }

    #[test]
    fn stats_counts_and_ranks() {
        let s = stats("the cat and the hat and the bat", 2);
        assert_eq!(s.words, 8);
        assert_eq!(s.unique_words, 5);
        assert_eq!(s.longest_word, "the");
        assert_eq!(
            s.top,
            [
                WordCount {
                    word: "the".into(),
                    count: 3
                },
                WordCount {
                    word: "and".into(),
                    count: 2
                },
            ]
        );
    }

    #[test]
    fn stats_of_empty_text() {
        let s = stats("", 3);
        assert_eq!((s.words, s.unique_words, s.top.len()), (0, 0, 0));
        assert_eq!(s.longest_word, "");
    }
}
