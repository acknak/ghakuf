use std::error;
use std::fmt;
use std::fs;
use std::io;
use std::path;

/// An enum represents errors of SMF parser.
#[derive(Debug)]
pub enum ReadError {
    /// Reads tag error with invalid tag and file path at header.
    InvalidHeaderTag { tag: [u8; 4], path: path::PathBuf },
    /// Reads SMF identify code ([0x00, 0x00, 0x00, 0x06]) error at header.
    InvalidIdentifyCode { code: u32, path: path::PathBuf },
    /// Reads tag error with invalid tag and file path at track.
    InvalidTrackTag { tag: [u8; 4], path: path::PathBuf },
    /// Standard file IO error (std::io::Error)
    Io(io::Error),
    /// Reads SMF identify code ([0x00, 0x00, 0x00, 0x06]) error at header.
    UnknownMessageStatus { status: u8, path: path::PathBuf },
}

impl fmt::Display for ReadError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use reader::read_error::ReadError::*;
        match *self {
            InvalidHeaderTag { tag, ref path } => {
                write!(
                    f,
                    "Invalid header tag '{:?}' has found: {}",
                    tag,
                    fs::canonicalize(&path).unwrap().display()
                )
            }
            InvalidIdentifyCode { code, ref path } => {
                write!(
                    f,
                    "Invalid identify code '{}' has found at header: {}",
                    code,
                    fs::canonicalize(&path).unwrap().display()
                )
            }
            InvalidTrackTag { tag, ref path } => {
                write!(
                    f,
                    "Invalid track tag '{:?}' has found: {}",
                    tag,
                    fs::canonicalize(&path).unwrap().display()
                )
            }
            Io(ref err) => err.fmt(f),
            UnknownMessageStatus { status, ref path } => {
                write!(
                    f,
                    "Unknown message status '{:x}' has found: {}",
                    status,
                    fs::canonicalize(&path).unwrap().display()
                )
            }
        }
    }
}

impl error::Error for ReadError {
    fn description(&self) -> &str {
        use reader::read_error::ReadError::*;
        match *self {
            InvalidHeaderTag { .. } => "Invalid header tag has found. This file dosen't follow SMF format.",
            InvalidIdentifyCode { .. } => {
                concat!(
                    "Invalid SMF identify code has found at header.",
                    "This file dosen't follow SMF format."
                )
            }
            InvalidTrackTag { .. } => "Invalid track tag has found. This file dosen't follow SMF format.",
            ReadError::Io(ref err) => err.description(),
            UnknownMessageStatus { .. } => "Unknown message status has found. This file dosen't follow SMF format.",
        }
    }
}

impl From<io::Error> for ReadError {
    fn from(err: io::Error) -> ReadError {
        ReadError::Io(err)
    }
}
