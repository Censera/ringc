use std::{fmt, path::PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Location {
    pub line: usize,
    pub column: usize,
}

#[derive(Debug)]
pub enum Error {
    UnexpectedCharacter {
        character: char,
        at: Location,
    },
    UnexpectedToken {
        at: Location,
    },
    UnexpectedEnd {
        at: Location,
    },
    InvalidTime {
        value: String,
        at: Location,
    },
    InvalidDate {
        value: String,
        at: Location,
    },
    InvalidRange {
        from: String,
        to: String,
        at: Location,
    },
    UnsortedTimes {
        value: String,
        at: Location,
    },
    DuplicateSchedule {
        name: String,
        at: Location,
    },
    ScheduleNotFound {
        path: String,
        at: Location,
    },
    EmptySchedule,
    Read {
        message: String,
    },
}

impl Error {
    pub fn location(&self) -> Option<Location> {
        match self {
            Self::UnexpectedCharacter { at, .. }
            | Self::UnexpectedToken { at }
            | Self::UnexpectedEnd { at }
            | Self::InvalidTime { at, .. }
            | Self::InvalidDate { at, .. }
            | Self::InvalidRange { at, .. }
            | Self::UnsortedTimes { at, .. }
            | Self::DuplicateSchedule { at, .. }
            | Self::ScheduleNotFound { at, .. } => Some(*at),

            Self::EmptySchedule | Self::Read { .. } => None,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedCharacter { character, .. } => {
                write!(f, "unexpected character '{character}'")
            }

            Self::UnexpectedToken { .. } => write!(f, "unexpected token"),

            Self::UnexpectedEnd { .. } => write!(f, "unexpected end of file"),

            Self::InvalidTime { value, .. } => write!(f, "invalid time \"{value}\""),

            Self::InvalidDate { value, .. } => write!(f, "invalid date \"{value}\""),

            Self::InvalidRange { from, to, .. } => {
                write!(f, "invalid range \"{from}\" .. \"{to}\"")
            }

            Self::UnsortedTimes { value, .. } => {
                write!(f, "time \"{value}\" is not later than the previous time")
            }

            Self::DuplicateSchedule { name, .. } => {
                write!(f, "schedule \"{name}\" is defined more than once")
            }

            Self::ScheduleNotFound { path, .. } => {
                write!(f, "schedule file \"{path}\" not found")
            }

            Self::EmptySchedule => write!(f, "schedule has no times and no Disable"),

            Self::Read { message } => write!(f, "cannot read file: {message}"),
        }
    }
}

impl std::error::Error for Error {}

#[derive(Debug)]
pub struct Failure {
    pub file: PathBuf,
    pub error: Error,
}

impl fmt::Display for Failure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.error.location() {
            Some(at) => write!(
                f,
                "{}:{}:{}: {}",
                self.file.display(),
                at.line,
                at.column,
                self.error
            ),
            None => write!(f, "{}: {}", self.file.display(), self.error),
        }
    }
}

impl std::error::Error for Failure {}
