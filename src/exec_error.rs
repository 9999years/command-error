use std::fmt::Debug;
use std::fmt::Display;

use crate::CommandDisplay;
#[cfg(doc)]
use crate::CommandExt;
#[cfg(doc)]
use crate::OutputError;

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
    pub(crate) command: Box<dyn CommandDisplay>,
    pub(crate) inner: std::io::Error,
}

impl ExecError {
    /// Construct a new [`ExecError`].
    pub fn new(command: Box<dyn CommandDisplay>, inner: std::io::Error) -> Self {
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
        let program = self.command.program();
        #[cfg(feature = "shell-words")]
        let program = shell_words::quote(&program);
        // TODO: Should this contain an additional message like
        // "Is `program` installed and present in your `$PATH`?"
        write!(f, "Failed to execute `{program}`: {}", self.inner)
    }
}

impl std::error::Error for ExecError {}
