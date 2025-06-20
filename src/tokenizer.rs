//! Handles the tokenization of a GEDCOM file
use std::str::Chars;

/// The base enum of Token types making use of [GEDCOM Standard Release
/// 5.5.1](https://gedcom.io/specifications/ged551.pdf), p.11 `gedcom_line: level + delim +
/// [optional_xref_ID] + tag + [optional_line_value] + terminator`
#[derive(Clone, Debug, PartialEq)]
pub enum Token {
    /// The `level`, denoting the depth within the tree
    Level(u8),
    /// The `tag`, a four character code that distinguishes datatypes
    Tag(String),
    /// The value of the data: `optional_line_value`
    LineValue(String),
    /// The `optional_xref_ID` used throughout the file to refer to a particular face
    Pointer(String),
    /// A user-defined tag, always begins with an underscore
    CustomTag(String),
    /// End-of-file indicator
    EOF,
    /// The initial token value, indicating nothing
    None,
}

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
    #[must_use]
    pub fn done(&self) -> bool {
        self.current_token == Token::EOF
    }

    /// Loads the next token into state
    pub fn next_token(&mut self) {
        if self.current_char == '\0' {
            self.current_token = Token::EOF;
            return;
        }

        // level number is at the start of each line.
        if self.current_char == '\r' {
            self.next_char();
        }
        if self.current_char == '\n' {
            self.next_char();

            self.current_token = Token::Level(self.extract_number());
            self.line += 1;
            return;
        }

        self.skip_whitespace();

        // handle tag with trailing whitespace
        if self.current_char == '\n' {
            self.next_token();
            return;
        }

        self.current_token = match self.current_token {
            Token::Level(_) => {
                if self.current_char == '@' {
                    Token::Pointer(self.extract_word())
                } else if self.current_char == '_' {
                    Token::CustomTag(self.extract_word())
                } else {
                    Token::Tag(self.extract_word())
                }
            }
            Token::Pointer(_) => Token::Tag(self.extract_word()),
            Token::Tag(_) | Token::CustomTag(_) => Token::LineValue(self.extract_value()),
            _ => panic!(
                "line {}: Tokenization error! {:?}",
                self.line, self.current_token
            ),
        };
    }

    /// Like `next_token`, but returns a clone of the token you are popping.
    pub fn take_token(&mut self) -> Token {
        let current_token = self.current_token.clone();
        self.next_token();
        return current_token;
    }

    fn next_char(&mut self) {
        self.current_char = self.chars.next().unwrap_or('\0');
    }

    fn extract_number(&mut self) -> u8 {
        self.skip_whitespace();
        let mut digits: Vec<char> = Vec::new();
        while self.current_char.is_digit(10) {
            digits.push(self.current_char);
            self.next_char();
        }

        digits.iter().collect::<String>().parse::<u8>().unwrap()
    }

    fn extract_word(&mut self) -> String {
        let mut letters: Vec<char> = Vec::new();
        while !self.current_char.is_whitespace() && self.current_char != '\0' {
            letters.push(self.current_char);
            self.next_char();
        }

        letters.iter().collect::<String>()
    }

    fn extract_value(&mut self) -> String {
        let mut letters: Vec<char> = Vec::new();
        while self.current_char != '\n' && self.current_char != '\r' {
            letters.push(self.current_char);
            self.next_char();
        }

        letters.iter().collect::<String>()
    }

    fn skip_whitespace(&mut self) {
        while self.is_nonnewline_whitespace() {
            self.next_char();
        }
    }

    fn is_nonnewline_whitespace(&self) -> bool {
        let is_zero_width_space = self.current_char as u32 == 65279_u32;
        let not_a_newline = self.current_char != '\n';
        (self.current_char.is_whitespace() || is_zero_width_space) && not_a_newline
    }

    /// Debug function displaying GEDCOM line number of error message.
    pub fn debug(&self) -> String {
        format!("line {}:", self.line)
    }

    /// Grabs and returns to the end of the current line as a String
    pub fn take_line_value(&mut self) -> String {
        let mut value = String::from("");
        self.next_token();

        match &self.current_token {
            Token::LineValue(val) => {
                value = val.to_string();
                self.next_token();
            }
            // gracefully handle an attempt to take a value from a valueless line
            Token::Level(_) => (),
            _ => panic!(
                "{} Expected LineValue, found {:?}",
                self.debug(),
                self.current_token
            ),
        }
        value
    }

    /// Takes the value of the current line including handling
    /// multi-line values from CONT & CONC tags.
    pub fn take_continued_text(&mut self, level: u8) -> String {
        let mut value = self.take_line_value();

        loop {
            if let Token::Level(cur_level) = self.current_token {
                if cur_level <= level {
                    break;
                }
            }
            match &self.current_token {
                Token::Tag(tag) => match tag.as_str() {
                    "CONT" => {
                        value.push('\n');
                        value.push_str(&self.take_line_value())
                    }
                    "CONC" => {
                        // value.push(' ');
                        value.push_str(&self.take_line_value())
                    }
                    _ => panic!("{} Unhandled Continuation Tag: {}", self.debug(), tag),
                },
                Token::Level(_) => self.next_token(),
                _ => panic!(
                    "{} Unhandled Continuation Token: {:?}",
                    self.debug(),
                    self.current_token
                ),
            }
        }
        value
    }
}
