// SPDX-License-Identifier: LGPL-2.1-or-later
// Copyright 2018 Jacob Lifshay

use std::borrow::Borrow;
use std::iter::Iterator;

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
enum CharClass {
    Uppercase,
    OtherIdentifier,
    Number,
    WordSeparator,
}

impl From<char> for CharClass {
    fn from(v: char) -> CharClass {
        match v {
            'A'...'Z' => CharClass::Uppercase,
            'a'...'z' => CharClass::OtherIdentifier,
            '0'...'9' => CharClass::Number,
            _ => CharClass::WordSeparator,
        }
    }
}

pub struct WordIterator<'a> {
    word: Option<&'a str>,
    words: &'a str,
}

impl<'a> WordIterator<'a> {
    pub fn new(words: &'a str) -> Self {
        WordIterator { word: None, words }
    }
}

impl<'a> Iterator for WordIterator<'a> {
    type Item = &'a str;
    fn next(&mut self) -> Option<&'a str> {
        let mut word_start = None;
        let mut last_char_class = CharClass::WordSeparator;
        for (i, ch) in self.words.char_indices() {
            let current_char_class = CharClass::from(ch);
            if word_start.is_some() {
                match current_char_class {
                    CharClass::WordSeparator => {
                        self.word = Some(&self.words[word_start.unwrap()..i]);
                        self.words = &self.words[i..];
                        return self.word;
                    }
                    CharClass::Uppercase => {
                        if last_char_class != CharClass::Uppercase
                            && last_char_class != CharClass::Number
                        {
                            self.word = Some(&self.words[word_start.unwrap()..i]);
                            self.words = &self.words[i..];
                            return self.word;
                        }
                        if self.words[i..].chars().nth(1).map(CharClass::from)
                            == Some(CharClass::OtherIdentifier)
                        {
                            self.word = Some(&self.words[word_start.unwrap()..i]);
                            self.words = &self.words[i..];
                            return self.word;
                        }
                    }
                    _ => {}
                }
            } else if current_char_class != CharClass::WordSeparator {
                word_start = Some(i);
            }
            last_char_class = current_char_class;
        }
        if let Some(word_start) = word_start {
            self.word = Some(&self.words[word_start..]);
        } else {
            self.word = None;
        }
        self.words = "";
        self.word
    }
}

pub const RUST_RESERVED_WORDS: &[&str] = &[
    "_", "Self", "abstract", "alignof", "as", "become", "box", "break", "const", "continue",
    "crate", "do", "else", "enum", "extern", "false", "final", "fn", "for", "if", "impl", "in",
    "let", "loop", "macro", "match", "mod", "move", "mut", "offsetof", "override", "priv", "proc",
    "pub", "pure", "ref", "return", "self", "sizeof", "static", "struct", "super", "trait", "true",
    "type", "typeof", "unsafe", "unsized", "use", "virtual", "where", "while", "yield",
];

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum CharacterCase {
    Upper,
    Lower,
}

impl CharacterCase {
    pub fn convert_ascii_case<T: Into<String>>(self, string: T) -> String {
        let mut retval = string.into();
        match self {
            CharacterCase::Upper => retval.make_ascii_uppercase(),
            CharacterCase::Lower => retval.make_ascii_lowercase(),
        }
        retval
    }
    pub fn convert_initial_ascii_case<T: Into<String>>(self, string: T) -> String {
        let mut retval = string.into();
        if let Some(first) = retval.get_mut(0..1) {
            match self {
                CharacterCase::Upper => first.make_ascii_uppercase(),
                CharacterCase::Lower => first.make_ascii_lowercase(),
            }
        }
        retval
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub enum NameFormat {
    SnakeCase,
    ScreamingSnakeCase,
    CamelCase,
}

impl NameFormat {
    pub fn word_separator(self) -> &'static str {
        match self {
            NameFormat::SnakeCase | NameFormat::ScreamingSnakeCase => "_",
            NameFormat::CamelCase => "",
        }
    }
    pub fn word_initial_char_case(self) -> CharacterCase {
        match self {
            NameFormat::CamelCase | NameFormat::ScreamingSnakeCase => CharacterCase::Upper,
            NameFormat::SnakeCase => CharacterCase::Lower,
        }
    }
    pub fn word_char_case(self) -> CharacterCase {
        match self {
            NameFormat::ScreamingSnakeCase => CharacterCase::Upper,
            NameFormat::CamelCase | NameFormat::SnakeCase => CharacterCase::Lower,
        }
    }
    pub fn name_from_words<T: Borrow<str>, I: Iterator<Item = T>>(
        self,
        words: I,
    ) -> Option<String> {
        let mut retval: Option<String> = None;
        for word in words {
            let word = word.borrow();
            let word = self.word_char_case().convert_ascii_case(word);
            let word = self
                .word_initial_char_case()
                .convert_initial_ascii_case(word);
            retval = Some(if let Some(mut s) = retval {
                s + self.word_separator() + &word
            } else {
                word
            });
        }
        let retval = retval?;
        for &reserved_word in RUST_RESERVED_WORDS {
            if retval == reserved_word {
                return Some(retval + "_");
            }
        }
        Some(retval)
    }
}
