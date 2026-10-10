use crate::error::{Error, Location};

#[derive(Debug, PartialEq, Eq)]
pub enum Kind {
    /// A run of letters, digits, `_`, `-` and `:`. Names, weekdays, dates, and times.
    Word(String),
    LeftBrace,
    RightBrace,
    Range,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Token {
    pub kind: Kind,
    pub at: Location,
}

fn is_word_character(character: char) -> bool {
    character.is_ascii_alphanumeric() || matches!(character, '_' | '-' | ':')
}

pub fn lex(source: &str) -> Result<Vec<Token>, Error> {
    let mut tokens = Vec::new();
    let mut chars = source.chars().peekable();
    let mut line = 1;
    let mut column = 1;

    while let Some(character) = chars.next() {
        let at = Location { line, column };
        column += 1;

        match character {
            '\n' => {
                line += 1;
                column = 1;
            }

            character if character.is_whitespace() => {}

            '{' => tokens.push(Token {
                kind: Kind::LeftBrace,
                at,
            }),

            '}' => tokens.push(Token {
                kind: Kind::RightBrace,
                at,
            }),

            '.' => {
                if chars.peek() == Some(&'.') {
                    chars.next();
                    column += 1;
                    tokens.push(Token {
                        kind: Kind::Range,
                        at,
                    });
                } else {
                    return Err(Error::UnexpectedCharacter { character, at });
                }
            }

            '/' => {
                if chars.peek() == Some(&'/') {
                    while let Some(&next) = chars.peek() {
                        if next == '\n' {
                            break;
                        }

                        chars.next();
                        column += 1;
                    }
                } else {
                    return Err(Error::UnexpectedCharacter { character, at });
                }
            }

            character if is_word_character(character) => {
                let mut value = String::from(character);

                while let Some(&next) = chars.peek() {
                    if !is_word_character(next) {
                        break;
                    }

                    value.push(next);
                    chars.next();
                    column += 1;
                }

                tokens.push(Token {
                    kind: Kind::Word(value),
                    at,
                });
            }

            character => return Err(Error::UnexpectedCharacter { character, at }),
        }
    }

    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(line: usize, column: usize) -> Location {
        Location { line, column }
    }

    #[test]
    fn tokens_carry_locations() {
        let tokens = lex("Standard {\n    2026-9-7 .. 2027-6-26\n}").unwrap();

        assert_eq!(
            tokens,
            vec![
                Token {
                    kind: Kind::Word("Standard".into()),
                    at: at(1, 1)
                },
                Token {
                    kind: Kind::LeftBrace,
                    at: at(1, 10)
                },
                Token {
                    kind: Kind::Word("2026-9-7".into()),
                    at: at(2, 5)
                },
                Token {
                    kind: Kind::Range,
                    at: at(2, 14)
                },
                Token {
                    kind: Kind::Word("2027-6-26".into()),
                    at: at(2, 17)
                },
                Token {
                    kind: Kind::RightBrace,
                    at: at(3, 1)
                },
            ]
        );
    }

    #[test]
    fn range_without_spaces() {
        let tokens = lex("2026-9-7..2027-6-26").unwrap();

        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[1].kind, Kind::Range);
    }

    #[test]
    fn comments_are_skipped_and_locations_stay_right() {
        let tokens = lex("// note\n08:00 // another\n09:00").unwrap();

        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].at, at(2, 1));
        assert_eq!(tokens[1].at, at(3, 1));
    }

    #[test]
    fn commas_are_rejected() {
        assert!(matches!(
            lex("08:00, 09:00"),
            Err(Error::UnexpectedCharacter {
                character: ',',
                at: Location { line: 1, column: 6 }
            })
        ));
    }

    #[test]
    fn quotes_are_rejected() {
        assert!(matches!(
            lex("\"08:00\""),
            Err(Error::UnexpectedCharacter { character: '"', .. })
        ));
    }

    #[test]
    fn wildcards_are_rejected() {
        assert!(matches!(
            lex("*-03-31"),
            Err(Error::UnexpectedCharacter { character: '*', .. })
        ));
    }

    #[test]
    fn single_dot_and_single_slash_are_rejected() {
        assert!(lex("2026-9-7 . 2027-6-26").is_err());
        assert!(lex("/ nope").is_err());
    }
}
