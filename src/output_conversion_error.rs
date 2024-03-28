use std::fmt::Debug;
use std::fmt::Display;

#[cfg(doc)]
use std::process::Command;
#[cfg(doc)]
use std::process::Output;

#[cfg(doc)]
use utf8_command::Utf8Output;

use crate::CommandDisplay;

#[cfg(doc)]
use crate::CommandExt;

/// An error produced when attempting to convert [`Command`] [`Output`] to a custom format (such as
/// [`Utf8Output`]).
///
/// Produced by methods like [`CommandExt::output_checked_with`] and
/// [`CommandExt::output_checked_utf8`].
///
/// ```
/// # use pretty_assertions::assert_eq;
/// # use indoc::indoc;
/// # use std::process::Command;
/// # use std::process::Output;
/// # use std::process::ExitStatus;
/// # use command_error::Utf8ProgramAndArgs;
/// # use command_error::CommandDisplay;
/// # use command_error::OutputConversionError;
/// let mut command = Command::new("sh");
/// command.args(["-c", "echo puppy doggy"]);
/// let displayed: Utf8ProgramAndArgs = (&command).into();
/// let mut output = command.output().unwrap();
/// output.stdout[5] = 0xc0; // Invalid UTF-8 byte.
/// let inner: Result<utf8_command::Utf8Output, _> = output.try_into();
/// let error = OutputConversionError::new(
///     Box::new(displayed),
///     Box::new(inner.unwrap_err())
/// );
/// assert_eq!(
///     error.to_string(),
///     "Failed to convert `sh` output: \
///     Stdout contained invalid utf-8 sequence of 1 bytes from index 5: \
///     \"puppy�doggy\\n\""
/// );
/// ```
pub struct OutputConversionError {
    pub(crate) command: Box<dyn CommandDisplay + Send + Sync>,
    pub(crate) inner: Box<dyn Display + Send + Sync>,
}

impl OutputConversionError {
    /// Construct a new [`OutputConversionError`].
    pub fn new(
        command: Box<dyn CommandDisplay + Send + Sync>,
        inner: Box<dyn Display + Send + Sync>,
    ) -> Self {
        Self { command, inner }
    }
}

impl Debug for OutputConversionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OutputConversionError")
            .field("program", &self.command.program())
            .field("inner", &self.inner.to_string())
            .finish()
    }
}

impl Display for OutputConversionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Failed to convert `{}` output: {}",
            self.command.program_quoted(),
            self.inner
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use static_assertions::assert_impl_all;

    assert_impl_all!(OutputConversionError: Send, Sync);
}
