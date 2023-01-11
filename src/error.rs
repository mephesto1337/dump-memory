use std::fmt;
use std::io;
use std::num::ParseIntError;

/// Errors for this crate
#[derive(Debug)]
pub enum Error {
    /// Underlying I/O error
    IO(io::Error),

    /// Cannot parse integer
    ParseIntError(ParseIntError),

    /// Missing field in region parsing
    MissingRegionField(&'static str),

    /// Malformed field in region parsing
    MalformedRegionField { field: &'static str, value: String },

    /// Region not found
    RegionNotFound { start: usize, end: usize },

    /// Ptrace error
    Ptrace(io::Error),
}

pub type Result<T> = ::std::result::Result<T, Error>;

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IO(ref e) => fmt::Display::fmt(e, f),
            Self::ParseIntError(ref e) => fmt::Display::fmt(e, f),
            Self::MissingRegionField(s) => write!(f, "Missing {} field in region", s),
            Self::MalformedRegionField { field, ref value } => {
                write!(f, "Malformed field {} in region: {:?}", field, value)
            }
            Self::RegionNotFound { start, end } => {
                write!(f, "Region mapped with 0x{:x}..0x{:x} not found", start, end)
            }
            Self::Ptrace(ref e) => write!(f, "ptrace error: {}", e),
        }
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Self::IO(e)
    }
}

impl From<ParseIntError> for Error {
    fn from(e: ParseIntError) -> Self {
        Self::ParseIntError(e)
    }
}
