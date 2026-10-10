use crate::error::Error;

#[derive(Debug, PartialEq, Eq)]
pub enum Token {
    String(String),
    Identifier(String),
    Comma,
    LeftBrace,
    RightBrace,
    Range,
}

pub fn lex(source: &str) -> Result<Vec<Token>, Error> {
    let mut tokens = Vec::new();
    let mut chars = source.char_indices().peekable();

    while let Some((position, character)) = chars.next() {
        match character {
            '"' => {
                let mut value = String::new();
                let mut terminated = false;

                for (_, character) in chars.by_ref() {
                    if character == '"' {
                        terminated = true;
                        break;
                    }

                    value.push(character);
                }

                if !terminated {
                    return Err(Error::UnterminatedString { position });
                }

                tokens.push(Token::String(value));
            }

            ',' => tokens.push(Token::Comma),

            '{' => tokens.push(Token::LeftBrace),

            '}' => tokens.push(Token::RightBrace),

            '.' => {
                if chars.peek().map(|(_, c)| *c) == Some('.') {
                    chars.next();
                    tokens.push(Token::Range);
                } else {
                    return Err(Error::UnexpectedCharacter {
                        character,
                        position,
                    });
                }
            }

            '/' => {
                if chars.peek().map(|(_, c)| *c) == Some('/') {
                    chars.next();

                    for (_, character) in chars.by_ref() {
                        if character == '\n' {
                            break;
                        }
                    }
                } else {
                    return Err(Error::UnexpectedCharacter {
                        character,
                        position,
                    });
                }
            }

            character if character.is_ascii_alphabetic() || character == '_' => {
                let mut value = String::from(character);

                while let Some((_, character)) = chars.peek() {
                    if character.is_ascii_alphanumeric() || *character == '_' {
                        value.push(*character);
                        chars.next();
                    } else {
                        break;
                    }
                }

                tokens.push(Token::Identifier(value));
            }

            character if character.is_whitespace() => {}

            character => {
                return Err(Error::UnexpectedCharacter {
                    character,
                    position,
                });
            }
        }
    }

    Ok(tokens)
}
