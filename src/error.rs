use std::fmt;

#[derive(Debug)]
pub enum Error {
    UnexpectedCharacter { character: char, position: usize },
    UnterminatedString { position: usize },
    InvalidTime { value: String, position: usize },
    InvalidDate { value: String, position: usize },
    InvalidRange {
        from: String,
        to: String,
        position: usize,
    },
    UnexpectedToken { position: usize },
    NoSchedule,
    AmbiguousSchedule,
    ScheduleNotFound {
        name: String,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedCharacter {
                character,
                position,
            } => {
                write!(f, "unexpected character '{character}' at {position}")
            }

            Self::UnterminatedString { position } => {
                write!(f, "unterminated string at {position}")
            }

            Self::InvalidTime { value, position } => {
                write!(f, "invalid time \"{value}\" at {position}")
            }

            Self::InvalidDate { value, position } => {
                write!(f, "invalid date \"{value}\" at {position}")
            }

            Self::UnexpectedToken { position } => {
                write!(f, "unexpected token at {position}")
            }
            Self::NoSchedule => {
                write!(f, "no schedule is active")
            }

            Self::AmbiguousSchedule => {
                write!(f, "multiple schedules are active")
            }

            Self::ScheduleNotFound { name } => {
                write!(f, "schedule file \"{name}.bell\" not found")
            }
            
            Self::InvalidRange {
                from,
                to,
                position,
            } => {
                write!(f, "invalid range \"{from}\" .. \"{to}\" at {position}")
            }
        }
    }
}

impl std::error::Error for Error {}
