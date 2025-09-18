use std::fmt::Debug;
use std::fmt::Display;

use crate::CommandDisplay;
#[cfg(doc)]
use crate::CommandExt;
#[cfg(doc)]
use crate::OutputError;
#[cfg(feature = "miette")]
use miette::Diagnostic;

/// An error from failing to execute a command. Produced by [`CommandExt`].
///
/// This is a command that fails to start, rather than a command that exits with a non-zero status
/// or similar, like [`OutputError`].
///
/// ```
/// # use pretty_assertions::assert_eq;
/// # use std::process::Command;
/// # use command_error::Utf8ProgramAndArgs;
/// # use command_error::CommandDisplay;
/// # use command_error::ExecError;
/// let mut command = Command::new("echo");
/// command.arg("puppy doggy");
/// let displayed: Utf8ProgramAndArgs = (&command).into();
/// let error = ExecError::new(
///     Box::new(displayed),
///     std::io::Error::new(
///         std::io::ErrorKind::NotFound,
///         "File not found (os error 2)"
///     ),
/// );
/// assert_eq!(
///     error.to_string(),
///     "Failed to execute `echo`: File not found (os error 2)"
/// );
/// ```
pub struct ExecError {
    command: Box<dyn CommandDisplay + Send + Sync>,
    inner: std::io::Error,
}

impl ExecError {
    /// Construct a new [`ExecError`].
    pub fn new(command: Box<dyn CommandDisplay + Send + Sync>, inner: std::io::Error) -> Self {
        Self { command, inner }
    }
}

impl Debug for ExecError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ExecError")
            .field("program", &self.command.program())
            .field("inner", &self.inner)
            .finish()
    }
}

impl Display for ExecError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Failed to execute `{}`: {}",
            self.command.program_quoted(),
            self.inner
        )
    }
}

impl std::error::Error for ExecError {}

#[cfg(feature = "miette")]
impl Diagnostic for ExecError {
    fn help<'a>(&'a self) -> Option<Box<dyn Display + 'a>> {
        Some(Box::new(format!(
            "Is {} installed and present on your $PATH?",
            self.command.program_quoted()
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use static_assertions::assert_impl_all;

    assert_impl_all!(ExecError: Send, Sync);
}
