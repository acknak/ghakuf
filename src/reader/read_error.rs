use std::error;
use std::fmt;
use std::fs;
use std::io;
use std::path;

#[derive(Debug)]
pub enum ReadError {
    Io(io::Error),
    SmfFormat(path::PathBuf),
}

impl fmt::Display for ReadError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ReadError::Io(ref err) => err.fmt(f),
            ReadError::SmfFormat(ref path) => {
                write!(
                    f,
                    "SMF Format Error: {}",
                    fs::canonicalize(path).unwrap_or(path.clone()).display()
                )
            }
        }
    }
}

impl error::Error for ReadError {
    fn description(&self) -> &str {
        match *self {
            ReadError::Io(ref err) => err.description(),
            ReadError::SmfFormat(_) => {
                "This file does not follow the format of Standard MIDI File."
            }
        }
    }
}

impl From<io::Error> for ReadError {
    fn from(err: io::Error) -> ReadError {
        ReadError::Io(err)
    }
}

impl From<path::PathBuf> for ReadError {
    fn from(path: path::PathBuf) -> ReadError {
        ReadError::SmfFormat(path)
    }
}
