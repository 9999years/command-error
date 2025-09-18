use std::fmt::Debug;
use std::fmt::Display;

#[cfg(doc)]
use crate::ChildExt;
use crate::CommandDisplay;
#[cfg(feature = "miette")]
use miette::Diagnostic;

/// An error from failing to wait for a command. Produced by [`ChildExt`].
///
/// ```
/// # use pretty_assertions::assert_eq;
/// # use std::process::Command;
/// # use command_error::Utf8ProgramAndArgs;
/// # use command_error::CommandDisplay;
/// # use command_error::WaitError;
/// let mut command = Command::new("echo");
/// command.arg("puppy doggy");
/// let displayed: Utf8ProgramAndArgs = (&command).into();
/// let error = WaitError::new(
///     Box::new(displayed),
///     std::io::Error::new(
///         std::io::ErrorKind::NotFound,
///         "File not found (os error 2)"
///     ),
/// );
/// assert_eq!(
///     error.to_string(),
///     "Failed to wait for `echo`: File not found (os error 2)"
/// );
/// ```
pub struct WaitError {
    command: Box<dyn CommandDisplay + Send + Sync>,
    inner: std::io::Error,
}

impl WaitError {
    /// Construct a new [`WaitError`].
    pub fn new(command: Box<dyn CommandDisplay + Send + Sync>, inner: std::io::Error) -> Self {
        Self { command, inner }
    }
}

impl Debug for WaitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WaitError")
            .field("program", &self.command.program())
            .field("inner", &self.inner)
            .finish()
    }
}

impl Display for WaitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Failed to wait for `{}`: {}",
            self.command.program_quoted(),
            self.inner
        )
    }
}

impl std::error::Error for WaitError {}

#[cfg(feature = "miette")]
impl Diagnostic for WaitError {}

#[cfg(test)]
mod tests {
    use super::*;
    use static_assertions::assert_impl_all;

    assert_impl_all!(WaitError: Send, Sync);
}
