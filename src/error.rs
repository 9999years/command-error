use std::fmt::Display;

#[cfg(doc)]
use std::process::Child;
#[cfg(doc)]
use std::process::Command;
#[cfg(doc)]
use std::process::Output;

use crate::output_conversion_error::OutputConversionError;
use crate::ExecError;
use crate::OutputError;
use crate::WaitError;

#[cfg(doc)]
use crate::CommandExt;
#[cfg(feature = "miette")]
use miette::Diagnostic;

/// An error produced by a [`Command`] failure.
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    /// An execution failure, when a [`Command`] fails to start or [`Child::wait`].
    Exec(ExecError),
    /// A failure to wait for a [`Command`].
    ///
    /// See: [`Child::wait`].
    Wait(WaitError),
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

impl Error {
    #[cfg(feature = "miette")]
    fn as_inner_diagnostic(&self) -> &(dyn Diagnostic + Send + Sync + 'static) {
        match self {
            Error::Exec(inner) => inner,
            Error::Wait(inner) => inner,
            Error::Output(inner) => inner,
            Error::Conversion(inner) => inner,
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Exec(error) => write!(f, "{}", error),
            Error::Wait(error) => write!(f, "{}", error),
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

impl From<WaitError> for Error {
    fn from(error: WaitError) -> Self {
        Self::Wait(error)
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

#[cfg(feature = "miette")]
impl Diagnostic for Error {
    fn code<'a>(&'a self) -> Option<Box<dyn Display + 'a>> {
        self.as_inner_diagnostic().code()
    }

    fn severity(&self) -> Option<miette::Severity> {
        self.as_inner_diagnostic().severity()
    }

    fn help<'a>(&'a self) -> Option<Box<dyn Display + 'a>> {
        self.as_inner_diagnostic().help()
    }

    fn url<'a>(&'a self) -> Option<Box<dyn Display + 'a>> {
        self.as_inner_diagnostic().url()
    }

    fn source_code(&self) -> Option<&dyn miette::SourceCode> {
        self.as_inner_diagnostic().source_code()
    }

    fn labels(&self) -> Option<Box<dyn Iterator<Item = miette::LabeledSpan> + '_>> {
        self.as_inner_diagnostic().labels()
    }

    fn related<'a>(&'a self) -> Option<Box<dyn Iterator<Item = &'a dyn Diagnostic> + 'a>> {
        self.as_inner_diagnostic().related()
    }

    fn diagnostic_source(&self) -> Option<&dyn Diagnostic> {
        self.as_inner_diagnostic().diagnostic_source()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use static_assertions::assert_impl_all;

    assert_impl_all!(Error: Send, Sync);
}
