//! IronXL: the criteria grammar and the wildcard matcher of the engine, for a caller outside the engine.
//!
//! `Criterion` reads a criteria text as `SUMIF`, `COUNTIF`, `AVERAGEIF`, and the `*IFS` functions read it.
//! `Wildcard` matches a text with an Excel wildcard pattern as the lookup functions do. A caller that uses them
//! agrees with the formulas of the same workbook.

#[cfg(target_arch = "wasm32")]
use regex_lite as regex;

use crate::{
    calc_result::CalcResult,
    expressions::{token::Error, types::CellReferenceIndex},
    formatter::dates::DateSystem,
    functions::util::{build_text_criteria, from_wildcard_to_regex},
    locale::Locale,
};

/// The value of one cell, as a criterion tests it.
#[derive(Clone, Debug, PartialEq)]
pub enum CriteriaValue<'a> {
    /// A cell without a value.
    Empty,
    Number(f64),
    Text(&'a str),
    Boolean(bool),
    Error(Error),
}

/// The criterion of one criteria text, such as `"> 1000"`, `"n*"`, or `"<>"`.
pub struct Criterion<'a> {
    test: Box<dyn Fn(&CalcResult) -> bool + 'a>,
}

impl<'a> Criterion<'a> {
    /// The criterion of `criteria`. Every text is a criterion, as in Excel: a text that does not read as a number,
    /// a date, a Boolean, an error, or a wildcard pattern compares as text. `locale` and `date_system` read a date,
    /// such as `"> 1/1/2024"`, as the formulas of a model with the same settings do.
    pub fn new(criteria: &'a str, locale: &'a Locale, date_system: DateSystem) -> Self {
        Self {
            test: build_text_criteria(criteria, locale, date_system),
        }
    }

    /// True when `value` meets the criterion.
    pub fn matches(&self, value: &CriteriaValue) -> bool {
        let value = match value {
            CriteriaValue::Empty => CalcResult::EmptyCell,
            CriteriaValue::Number(number) => CalcResult::Number(*number),
            CriteriaValue::Text(text) => CalcResult::String((*text).to_owned()),
            CriteriaValue::Boolean(flag) => CalcResult::Boolean(*flag),
            CriteriaValue::Error(error) => CalcResult::Error {
                error: error.clone(),
                origin: CellReferenceIndex {
                    sheet: 0,
                    column: 1,
                    row: 1,
                },
                message: String::new(),
            },
        };
        (self.test)(&value)
    }
}

/// An Excel wildcard pattern: `?` matches one character, `*` matches any text, and `~` makes the next character
/// literal. The match ignores case, as in Excel.
pub struct Wildcard {
    regex: regex::Regex,
}

impl Wildcard {
    /// The matcher of `pattern`. With `exact`, the pattern must match the whole text; else a part of the text.
    /// The error is the message of a pattern that the regular expression engine does not take, such as one that
    /// is too large.
    pub fn new(pattern: &str, exact: bool) -> Result<Self, String> {
        from_wildcard_to_regex(&pattern.to_lowercase(), exact)
            .map(|regex| Self { regex })
            .map_err(|error| error.to_string())
    }

    /// True when the pattern matches `text`.
    pub fn is_match(&self, text: &str) -> bool {
        self.regex.is_match(&text.to_lowercase())
    }
}
