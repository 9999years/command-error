use std::fmt::Display;

use crate::output_conversion_error::OutputConversionError;
use crate::ExecError;
use crate::OutputError;

#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    Exec(ExecError),
    Output(OutputError),
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
