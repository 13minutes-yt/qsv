#![macro_use]
use std::borrow::ToOwned;
use std::fmt;
use std::io;
use std::process::{ExitCode, Termination};

macro_rules! wout {
    ($($arg:tt)*) => ({
        use std::io::Write;
        (writeln!(&mut ::std::io::stdout(), $($arg)*)).unwrap();
    });
}

macro_rules! werr {
    ($($arg:tt)*) => ({
        use std::io::Write;
        use log::error;
        error!("{}", $($arg)*);
        (writeln!(&mut ::std::io::stderr(), $($arg)*)).unwrap();
    });
}

macro_rules! fail {
    ($e:expr) => {{
        use log::error;
        let err = ::std::convert::From::from($e);
        error!("{err}");
        Err(err)
    }};
}

macro_rules! fail_format {
    ($($t:tt)*) => {{
        use log::error;
        use crate::CliError;
        let err = format!($($t)*);
        error!("{err}");
        Err(CliError::Other(err))
    }};
}

#[repr(u8)]
pub enum QsvExitCode {
    Good = 0,
    Bad = 1,
    IncorrectUsage = 2,
    Abort = 255,
}

impl Termination for QsvExitCode {
    fn report(self) -> ExitCode {
        ExitCode::from(self as u8)
    }
}

pub type CliResult<T> = Result<T, CliError>;

#[derive(Debug)]
pub enum CliError {
    Flag(docopt::Error),
    Csv(csv::Error),
    Io(io::Error),
    NoMatch(),
    Other(String),
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            CliError::Flag(ref e) => e.fmt(f),
            CliError::Csv(ref e) => e.fmt(f),
            CliError::Io(ref e) => e.fmt(f),
            CliError::NoMatch() => f.write_str("no_match"),
            CliError::Other(ref s) => f.write_str(s),
        }
    }
}

impl From<docopt::Error> for CliError {
    fn from(err: docopt::Error) -> CliError {
        CliError::Flag(err)
    }
}

impl From<csv::Error> for CliError {
    fn from(err: csv::Error) -> CliError {
        if !err.is_io_error() {
            return CliError::Csv(err);
        }
        match err.into_kind() {
            csv::ErrorKind::Io(v) => From::from(v),
            _ => unreachable!(),
        }
    }
}

impl From<io::Error> for CliError {
    fn from(err: io::Error) -> CliError {
        CliError::Io(err)
    }
}

impl From<String> for CliError {
    fn from(err: String) -> CliError {
        CliError::Other(err)
    }
}

impl<'a> From<&'a str> for CliError {
    fn from(err: &'a str) -> CliError {
        CliError::Other(err.to_owned())
    }
}

impl From<regex::Error> for CliError {
    fn from(err: regex::Error) -> CliError {
        CliError::Other(format!("Regex error: {err:?}"))
    }
}
