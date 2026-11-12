// Grammar of Excel's structured references:
//
//  structured_reference -> [table_name] "[" body "]"
//  body                 -> ""                                   Table1[]
//                        | specifier                            Table1[#Totals]
//                        | "@" [column | "[" columns "]"]       Table1[@Amount], [@[Unit Price]], [@[Jan]:[Dec]]
//                        | simple_column                        Table1[Amount]
//                        | item ("," item)*                     Table1[[#Data],[#Totals],[Amount]]
//  item                 -> "[" specifier "]" | "[" column "]" [":" "[" column "]"]
//  specifier            -> "#All" | "#Data" | "#Headers" | "#Totals" | "#This Row"
//
// A specifier compares without case. In a column name, the `'` escapes the next character: `'[`, `']`, `'#`, `'@`,
// and `''`. Two specifiers are valid only as `[#Headers],[#Data]` and `[#Data],[#Totals]`.
// A reference without a table name, such as `[@Amount]` or `[Amount]`, has an empty table name.

use crate::expressions::token::TokenType;
use crate::expressions::token::{TableReference, TableSpecifier};

use super::Result;
use super::{Lexer, LexerError};

/// One item of a structured reference in inner brackets.
enum Item {
    Specifier(TableSpecifier),
    Columns(TableReference),
}

impl<'a> Lexer<'a> {
    fn structured_error(&self, position: usize) -> LexerError {
        LexerError {
            position,
            message: "Invalid structured reference".to_string(),
        }
    }

    fn skip_spaces(&mut self) {
        while self.position < self.len && self.chars[self.position] == ' ' {
            self.position += 1;
        }
    }

    fn take_char(&mut self, expected: char) -> Result<()> {
        if self.position < self.len && self.chars[self.position] == expected {
            self.position += 1;
            Ok(())
        } else {
            Err(self.structured_error(self.position))
        }
    }

    /// Reads a specifier from `#` to the `]` that closes it, and consumes that `]`.
    fn consume_table_specifier(&mut self) -> Result<TableSpecifier> {
        let start = self.position;
        let mut end = start;
        while end < self.len && self.chars[end] != ']' {
            end += 1;
        }
        let text: String = self.chars[start..end].iter().collect();
        let specifier = match text.to_lowercase().as_str() {
            "#all" => TableSpecifier::All,
            "#data" => TableSpecifier::Data,
            "#headers" => TableSpecifier::Headers,
            "#totals" => TableSpecifier::Totals,
            "#this row" => TableSpecifier::ThisRow,
            _ => return Err(self.structured_error(start)),
        };
        self.position = end;
        self.take_char(']')?;
        Ok(specifier)
    }

    /// Reads a column name to the `]` that no `'` escapes, and consumes that `]`. An empty name is invalid.
    fn consume_column_name(&mut self) -> Result<String> {
        let start = self.position;
        let mut name = String::new();
        loop {
            match self.chars.get(self.position) {
                None => return Err(self.structured_error(start)),
                Some(']') => break,
                Some('[') => return Err(self.structured_error(self.position)),
                Some('\'') => {
                    self.position += 1;
                    match self.chars.get(self.position) {
                        Some(&escaped) => name.push(escaped),
                        None => return Err(self.structured_error(start)),
                    }
                }
                Some(&char) => name.push(char),
            }
            self.position += 1;
        }
        self.position += 1;
        if name.is_empty() {
            return Err(self.structured_error(start));
        }
        Ok(name)
    }

    /// Reads `[column]` or `[column]:[column]`, from the first `[`.
    fn consume_columns(&mut self) -> Result<TableReference> {
        self.take_char('[')?;
        let left = self.consume_column_name()?;
        let position = self.position;
        self.skip_spaces();
        if self.chars.get(self.position) == Some(&':') {
            self.position += 1;
            self.skip_spaces();
            self.take_char('[')?;
            let right = self.consume_column_name()?;
            Ok(TableReference::RangeReference((left, right)))
        } else {
            self.position = position;
            Ok(TableReference::ColumnReference(left))
        }
    }

    /// Reads one item in inner brackets: `[#Data]`, `[Amount]`, or `[Jan]:[Dec]`.
    fn consume_item(&mut self) -> Result<Item> {
        if self.chars.get(self.position + 1) == Some(&'#') {
            self.take_char('[')?;
            Ok(Item::Specifier(self.consume_table_specifier()?))
        } else {
            Ok(Item::Columns(self.consume_columns()?))
        }
    }

    /// Reads the structured reference of the table `table_name` from its `[`. An empty `table_name` reads a reference
    /// without a table name.
    ///
    /// Possibilities:
    ///  1. `MyTable[]`, `MyTable[#Totals]`, `MyTable[#This Row]`
    ///  2. `MyTable[MyColumn]`, `MyTable[[My Column]]`, `MyTable[[Jan]:[Dec]]`
    ///  3. `MyTable[[#This Row],[My Column]]`, `MyTable[[#Totals],[Jan]:[Dec]]`
    ///  4. `MyTable[[#Headers],[#Data]]`, `MyTable[[#Data],[#Totals],[MyColumn]]`
    ///  5. `MyTable[@]`, `MyTable[@MyColumn]`, `MyTable[@[My Column]]`, `MyTable[@[Jan]:[Dec]]`
    pub(crate) fn consume_structured_reference(&mut self, table_name: &str) -> Result<TokenType> {
        let start = self.position;
        self.take_char('[')?;
        let token = |specifier, table_reference| TokenType::StructuredReference {
            table_name: table_name.to_string(),
            specifier,
            table_reference,
        };
        match self.chars.get(self.position) {
            Some(']') => {
                self.position += 1;
                return Ok(token(None, None));
            }
            Some('#') => {
                let specifier = self.consume_table_specifier()?;
                return Ok(token(Some(specifier), None));
            }
            Some('@') => {
                self.position += 1;
                let reference = match self.chars.get(self.position) {
                    Some(']') => {
                        self.position += 1;
                        None
                    }
                    Some('[') => {
                        let reference = self.consume_columns()?;
                        self.take_char(']')?;
                        Some(reference)
                    }
                    _ => Some(TableReference::ColumnReference(self.consume_column_name()?)),
                };
                return Ok(token(Some(TableSpecifier::ThisRow), reference));
            }
            Some('[') => {}
            _ => {
                let column = self.consume_column_name()?;
                return Ok(token(None, Some(TableReference::ColumnReference(column))));
            }
        }
        // A list of items in inner brackets, separated by commas
        let mut specifiers = Vec::new();
        let mut reference = None;
        loop {
            self.skip_spaces();
            match self.consume_item()? {
                // A specifier after a column is invalid
                Item::Specifier(specifier) if reference.is_none() => specifiers.push(specifier),
                Item::Columns(columns) if reference.is_none() => reference = Some(columns),
                _ => return Err(self.structured_error(start)),
            }
            self.skip_spaces();
            match self.chars.get(self.position) {
                Some(',') => self.position += 1,
                Some(']') => {
                    self.position += 1;
                    break;
                }
                _ => return Err(self.structured_error(self.position)),
            }
        }
        let specifier = match specifiers.as_slice() {
            [] => None,
            [one] => Some(one.clone()),
            [TableSpecifier::Headers, TableSpecifier::Data] => Some(TableSpecifier::HeadersData),
            [TableSpecifier::Data, TableSpecifier::Totals] => Some(TableSpecifier::DataTotals),
            _ => return Err(self.structured_error(start)),
        };
        Ok(token(specifier, reference))
    }

    /// Reads a structured reference without a table name, such as `[@Amount]`, from its `[`. Gives `None`, with the
    /// position unchanged, for text that is not one: `[]`, a number in brackets (the index of another workbook, as in
    /// `[1]Sheet1!A1`), and brackets that a name, a sheet, or `!` follows.
    pub(crate) fn consume_unqualified_structured_reference(&mut self) -> Option<TokenType> {
        let start = self.position;
        let token = match self.consume_structured_reference("") {
            Ok(token) => token,
            Err(_) => {
                self.position = start;
                return None;
            }
        };
        let empty_or_number = match &token {
            TokenType::StructuredReference {
                specifier: None,
                table_reference: None,
                ..
            } => true,
            TokenType::StructuredReference {
                specifier: None,
                table_reference: Some(TableReference::ColumnReference(name)),
                ..
            } => name.chars().all(|c| c.is_ascii_digit()),
            _ => false,
        };
        let followed_by_name = matches!(
            self.chars.get(self.position),
            Some(c) if c.is_alphanumeric() || matches!(c, '_' | '!' | '\'' | '.' | '\\' | '[')
        );
        if empty_or_number || followed_by_name {
            self.position = start;
            return None;
        }
        Some(token)
    }
}
