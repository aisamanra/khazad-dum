use std::fmt;
use std::path::PathBuf;

/// An Error is something which means we actively cannot compute
/// output and needed to stop at least some execution.
#[derive(Debug)]
pub struct Error {
    msg: String,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.msg.fmt(f)
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Error {
        Error {
            msg: format!("IO error: {}", err),
        }
    }
}

impl From<String> for Error {
    fn from(msg: String) -> Error {
        Error { msg }
    }
}

/// A Warning is a problem which doesn't stop most output from
/// happening but may be important.
#[derive(Debug)]
pub struct Warning {
    msg: String,
}

impl fmt::Display for Warning {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.msg.fmt(f)
    }
}

impl From<String> for Warning {
    fn from(msg: String) -> Warning {
        Warning { msg }
    }
}

/// A Line is a single line of output.
#[derive(Debug)]
pub struct Line {
    pub size: u64,
    pub md5: [u8; 16],
    pub sha256: [u8; 32],
    pub path: PathBuf,
}

/// An Item is a result which is delivered to the main loop: either a
/// successful line, an error, or a warning.
#[derive(Debug)]
pub enum Output {
    Warning(Warning),
    Error(Error),
    Entry(Line),
}

impl Output {
    /// Produce a warning
    pub fn warn(msg: impl Into<String>) -> Output {
        Output::Warning(Warning { msg: msg.into() })
    }
}
