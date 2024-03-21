use std::fmt::Debug;
use std::fmt::Display;

#[cfg(doc)]
use std::process::Command;
#[cfg(doc)]
use std::process::Output;

#[cfg(all(doc, feature = "utf8-command"))]
use utf8_command::Utf8Output;

use crate::CommandDisplay;

#[cfg(doc)]
use crate::CommandExt;

/// An error produced when attempting to convert [`Command`] [`Output`] to a custom format (such as
/// [`Utf8Output`]). Produced by methods like [`CommandExt::output_checked_with`] and
/// [`CommandExt::output_checked_utf8`].
pub struct OutputConversionError {
    pub(crate) command: Box<dyn CommandDisplay>,
    pub(crate) inner: Box<dyn Display>,
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
            self.command.program(),
            self.inner
        )
    }
}
