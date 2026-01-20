//! Processes character streams into tokens.
use crate::GedcomError;
use std::str::Chars;

/// The base enum of Token types making use of [GEDCOM Standard Release
/// 5.5.1](https://gedcom.io/specifications/ged551.pdf), p.11 `gedcom_line: level + delim +
/// [optional_xref_ID] + tag + [optional_line_value] + terminator`
#[derive(Clone, Debug, PartialEq)]
pub enum Token {
    /// The `level`, denoting the depth within the tree
    Level(u8),
    /// The `tag`, a four character code that distinguishes datatypes
    Tag(Box<str>),
    /// The value of the data: `optional_line_value`
    LineValue(Box<str>),
    /// The `optional_xref_ID` used throughout the file to refer to a particular face
    Pointer(Box<str>),
    /// A user-defined tag, always begins with an underscore
    CustomTag(Box<str>),
    /// End-of-file indicator
    EOF,
    /// The initial token value, indicating nothing
    None,
}

impl Token {
    /// Returns the string value of Tag tokens, or None for other token types.
    #[inline]
    #[must_use]
    pub fn as_tag_str(&self) -> Option<&str> {
        match self {
            Token::Tag(s) => Some(s),
            _ => None,
        }
    }

    /// Returns the string value of `LineValue` tokens, or None for other token types.
    #[inline]
    #[must_use]
    pub fn as_line_value_str(&self) -> Option<&str> {
        match self {
            Token::LineValue(s) => Some(s),
            _ => None,
        }
    }

    /// Returns the string value of Pointer tokens, or None for other token types.
    #[inline]
    #[must_use]
    pub fn as_pointer_str(&self) -> Option<&str> {
        match self {
            Token::Pointer(s) => Some(s),
            _ => None,
        }
    }

    /// Returns the string value of `CustomTag` tokens, or None for other token types.
    #[inline]
    #[must_use]
    pub fn as_custom_tag_str(&self) -> Option<&str> {
        match self {
            Token::CustomTag(s) => Some(s),
            _ => None,
        }
    }
}

/// Average length estimate for GEDCOM tags (most are 4 chars)
const TAG_CAPACITY: usize = 8;

/// Average length estimate for GEDCOM values
const VALUE_CAPACITY: usize = 64;

/// Average length estimate for xref pointers
const POINTER_CAPACITY: usize = 16;

/// The tokenizer that turns the GEDCOM characters into a list of tokens
pub struct Tokenizer<'a> {
    /// The active token type
    pub current_token: Token,
    /// Current character tokenizer is parsing
    current_char: char,
    /// An iterator of charaters of the GEDCOM file contents
    chars: Chars<'a>,
    /// The current line number of the file we are parsing
    pub line: u32,
}

impl<'a> Tokenizer<'a> {
    /// Creates a new tokenizer for a char interator of GEDCOM file contents
    #[must_use]
    pub fn new(chars: Chars<'a>) -> Tokenizer<'a> {
        Tokenizer {
            current_char: '\n',
            current_token: Token::None,
            chars,
            line: 0,
        }
    }

    /// Ends the tokenization
    #[inline]
    #[must_use]
    pub fn done(&self) -> bool {
        matches!(self.current_token, Token::EOF)
    }

    /// Loads the next token into state
    ///
    /// # Errors
    ///
    /// Returns a `GedcomError` if tokenization fails.
    pub fn next_token(&mut self) -> Result<(), GedcomError> {
        if self.current_char == '\0' {
            self.current_token = Token::EOF;
            return Ok(());
        }

        // level number is at the start of each line.
        if self.current_char == '\r' {
            self.next_char();
        }
        if self.current_char == '\n' {
            self.next_char();

            self.current_token = Token::Level(self.extract_number()?);
            self.line += 1;
            return Ok(());
        }

        self.skip_whitespace();

        // handle tag with trailing whitespace
        if self.current_char == '\n' {
            self.next_token()?;
            return Ok(());
        }

        self.current_token = match self.current_token {
            Token::Level(_) => {
                if self.current_char == '@' {
                    Token::Pointer(self.extract_word_with_capacity(POINTER_CAPACITY))
                } else if self.current_char == '_' {
                    Token::CustomTag(self.extract_word_with_capacity(TAG_CAPACITY))
                } else {
                    Token::Tag(self.extract_word_with_capacity(TAG_CAPACITY))
                }
            }
            Token::Pointer(_) => Token::Tag(self.extract_word_with_capacity(TAG_CAPACITY)),
            Token::Tag(_) | Token::CustomTag(_) => {
                Token::LineValue(self.extract_value_with_capacity(VALUE_CAPACITY))
            }
            _ => {
                return Err(GedcomError::ParseError {
                    line: self.line,
                    message: format!("Tokenization error! {:?}", self.current_token),
                })
            }
        };
        Ok(())
    }

    /// Like `next_token`, but returns a clone of the token you are popping.
    ///
    /// # Errors
    ///
    /// Returns a `GedcomError` if tokenization fails.
    pub fn take_token(&mut self) -> Result<Token, GedcomError> {
        let current_token = std::mem::replace(&mut self.current_token, Token::None);
        self.next_token()?;
        Ok(current_token)
    }

    #[inline]
    fn next_char(&mut self) {
        self.current_char = self.chars.next().unwrap_or('\0');
    }

    #[inline]
    fn extract_number(&mut self) -> Result<u8, GedcomError> {
        self.skip_whitespace();

        // Most levels are single digit (0-9), optimize for that case
        let first_digit = self.current_char;
        if !first_digit.is_ascii_digit() {
            return Err(GedcomError::ParseError {
                line: self.line,
                message: "Expected digit for level number".to_string(),
            });
        }

        self.next_char();

        // Check if there's a second digit
        if self.current_char.is_ascii_digit() {
            let second_digit = self.current_char;
            self.next_char();

            // Check for more digits (unlikely but handle gracefully)
            if self.current_char.is_ascii_digit() {
                // Collect remaining digits
                let mut digits = String::with_capacity(4);
                digits.push(first_digit);
                digits.push(second_digit);
                while self.current_char.is_ascii_digit() {
                    digits.push(self.current_char);
                    self.next_char();
                }
                return digits.parse::<u8>().map_err(|e| GedcomError::ParseError {
                    line: self.line,
                    message: format!("Failed to parse number: {e}"),
                });
            }

            // Two digit number
            let value = (first_digit as u8 - b'0') * 10 + (second_digit as u8 - b'0');
            return Ok(value);
        }

        // Single digit (most common case)
        Ok(first_digit as u8 - b'0')
    }

    #[inline]
    fn extract_word_with_capacity(&mut self, capacity: usize) -> Box<str> {
        let mut word = String::with_capacity(capacity);
        while !self.current_char.is_whitespace() && self.current_char != '\0' {
            word.push(self.current_char);
            self.next_char();
        }
        word.into_boxed_str()
    }

    #[inline]
    fn extract_value_with_capacity(&mut self, capacity: usize) -> Box<str> {
        let mut value = String::with_capacity(capacity);
        while self.current_char != '\n' && self.current_char != '\r' && self.current_char != '\0' {
            value.push(self.current_char);
            self.next_char();
        }
        value.into_boxed_str()
    }

    #[inline]
    fn skip_whitespace(&mut self) {
        while self.is_nonnewline_whitespace() {
            self.next_char();
        }
    }

    #[inline]
    fn is_nonnewline_whitespace(&self) -> bool {
        let c = self.current_char;
        // Check for BOM/zero-width space (U+FEFF = 65279)
        let is_zero_width_space = c as u32 == 65279_u32;
        let not_a_newline = c != '\n';
        (c.is_whitespace() || is_zero_width_space) && not_a_newline
    }

    /// Debug function displaying GEDCOM line number of error message.
    #[must_use]
    pub fn debug(&self) -> String {
        format!("line {}:", self.line)
    }

    /// Grabs and returns to the end of the current line as a String
    ///
    /// # Errors
    ///
    /// Returns a `GedcomError` if an unexpected line value is encountered.
    pub fn take_line_value(&mut self) -> Result<String, GedcomError> {
        self.next_token()?;

        match &self.current_token {
            Token::LineValue(val) => {
                let value = val.to_string();
                self.next_token()?;
                Ok(value)
            }
            // gracefully handle an attempt to take a value from a valueless line
            Token::Level(_) => Ok(String::new()),
            _ => Err(GedcomError::ParseError {
                line: self.line,
                message: format!("Expected LineValue, found {:?}", self.current_token),
            }),
        }
    }

    /// Takes the value of the current line including handling
    /// multi-line values from CONT & CONC tags.
    ///
    /// This function consumes `CONT` and `CONC` continuation tags at the next level,
    /// but stops gracefully when encountering any other tag, leaving the tokenizer
    /// positioned at that tag for subsequent parsing (e.g., by `parse_subset`).
    ///
    /// # Errors
    ///
    /// Returns a `GedcomError` if an unexpected token is encountered.
    pub fn take_continued_text(&mut self, level: u8) -> Result<String, GedcomError> {
        let mut value = self.take_line_value()?;

        loop {
            if let Token::Level(cur_level) = self.current_token {
                if cur_level <= level {
                    break;
                }
            }
            match &self.current_token {
                Token::Tag(tag) => match tag.as_ref() {
                    "CONT" => {
                        value.push('\n');
                        value.push_str(&self.take_line_value()?);
                    }
                    "CONC" => {
                        value.push_str(&self.take_line_value()?);
                    }
                    _ => {
                        // Non-continuation tag encountered; stop and leave it for parse_subset
                        break;
                    }
                },
                Token::Level(_) => self.next_token()?,
                Token::EOF => break,
                _ => {
                    return Err(GedcomError::ParseError {
                        line: self.line,
                        message: format!("Unhandled Continuation Token: {:?}", self.current_token),
                    })
                }
            }
        }
        Ok(value)
    }
}
