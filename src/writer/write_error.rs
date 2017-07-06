use std::error;
use std::fmt;
use std::fs;
use std::io;
use std::path;

#[derive(Debug)]
pub enum WriteError {
    Io(io::Error),
    SmfFormat(path::PathBuf),
}

impl fmt::Display for WriteError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            WriteError::Io(ref err) => err.fmt(f),
            WriteError::SmfFormat(ref path) => {
                write!(
                    f,
                    "SMF Format Error: {}",
                    fs::canonicalize(path).unwrap_or(path.clone()).display()
                )
            }
        }
    }
}

impl error::Error for WriteError {
    fn description(&self) -> &str {
        match *self {
            WriteError::Io(ref err) => err.description(),
            WriteError::SmfFormat(_) => {
                "This file does not follow the format of Standard MIDI File."
            }
        }
    }
}

impl From<io::Error> for WriteError {
    fn from(err: io::Error) -> WriteError {
        WriteError::Io(err)
    }
}

impl From<path::PathBuf> for WriteError {
    fn from(path: path::PathBuf) -> WriteError {
        WriteError::SmfFormat(path)
    }
}
