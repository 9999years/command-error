use std::fmt::Debug;
use std::fmt::Display;

use crate::CommandDisplay;

/// An error from failing to execute a command. Produced by [`CommandExt`].
///
/// This is a command that fails to start, rather than a command that exits with a non-zero status
/// or similar, like [`CommandError`].
pub struct ExecError {
    pub(crate) command: Box<dyn CommandDisplay>,
    pub(crate) inner: std::io::Error,
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
        // TODO: Should this contain an additional message like
        // "Is `program` installed and present in your `$PATH`?"
        write!(f, "Failed to execute `{}`: {}", self.command, self.inner)
    }
}

impl std::error::Error for ExecError {}
