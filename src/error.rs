use std::fmt::Display;

#[cfg(doc)]
use std::process::Command;
#[cfg(doc)]
use std::process::Output;

use crate::output_conversion_error::OutputConversionError;
use crate::ExecError;
use crate::OutputError;

#[cfg(doc)]
use crate::CommandExt;

/// An error produced by a [`Command`] failure.
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    /// An execution failure, when a [`Command`] fails to start.
    Exec(ExecError),
    /// An output failure, when a [`Command`] fails by returning a non-zero exit code (or in other
    /// cases, when custom validation logic is supplied in methods like
    /// [`CommandExt::output_checked_with`]).
    ///
    /// Note that this is also raised when non-capturing methods like
    /// [`CommandExt::status_checked`] fail.
    Output(OutputError),
    /// An output conversion error, when [`Output`] fails to convert to a custom format as
    /// requested by methods like [`CommandExt::output_checked_utf8`].
    Conversion(OutputConversionError),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Exec(error) => write!(f, "{}", error),
            Error::Output(error) => write!(f, "{}", error),
            Error::Conversion(error) => write!(f, "{}", error),
        }
    }
}

impl From<ExecError> for Error {
    fn from(error: ExecError) -> Self {
        Self::Exec(error)
    }
}

impl From<OutputError> for Error {
    fn from(error: OutputError) -> Self {
        Self::Output(error)
    }
}

impl From<OutputConversionError> for Error {
    fn from(error: OutputConversionError) -> Self {
        Self::Conversion(error)
    }
}

impl std::error::Error for Error {}
