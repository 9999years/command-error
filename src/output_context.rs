use std::borrow::Borrow;
use std::fmt::Debug;
use std::fmt::Display;
use std::process::ExitStatus;

#[cfg(doc)]
use std::process::Command;
#[cfg(doc)]
use std::process::Output;

#[cfg(doc)]
use crate::CommandExt;

use crate::CommandDisplay;
use crate::Error;
use crate::OutputError;
use crate::OutputLike;

/// [`Output`] combined with context about the [`Command`] that produced it.
///
/// This contains additional information about the command that was run (via a [`CommandDisplay`]
/// object) and can be used to construct error messages (for use in methods like
/// [`CommandExt::output_checked_as`]).
///
/// Note that because [`ExitStatus`] has a trivial implementation for [`OutputLike`] (where
/// [`stdout`][OutputLike::stdout] and [`stderr`][OutputLike::stderr] return empty strings), this
/// is also used as context for [`status`][`CommandExt::status_checked`] calls.
pub struct OutputContext<O> {
    pub(crate) output: O,
    pub(crate) command: Box<dyn CommandDisplay + Send + Sync>,
}

impl<O> OutputContext<O>
where
    O: OutputLike + Send + Sync + 'static,
{
    /// Get the [`OutputLike`] data contained in this context object.
    pub fn into_output(self) -> O {
        self.output
    }

    /// Get a reference to the [`OutputLike`] data contained in this context object.
    pub fn output(&self) -> &O {
        &self.output
    }

    /// Get the command's [`ExitStatus`].
    pub fn status(&self) -> ExitStatus {
        self.output.status()
    }

    /// Get a reference to the command contained in this context object, for use in error messages
    /// or diagnostics.
    pub fn command(&self) -> &(dyn CommandDisplay + Send + Sync) {
        self.command.borrow()
    }

    /// Get the command contained in this context object, for use in error messages
    /// or diagnostics.
    pub fn into_command(self) -> Box<dyn CommandDisplay> {
        self.command
    }

    /// Construct an error that indicates this command failed, containing information about the
    /// command and its output.
    ///
    /// See [`CommandExt`] for examples of the error format.
    pub fn error(self) -> Error {
        Error::from(OutputError::new(self.command, Box::new(self.output)))
    }

    /// Construct an error that indicates this command failed, containing information about the
    /// command, its output, and the provided error message.
    ///
    /// See [`CommandExt::output_checked_as`] for examples of the error format.
    pub fn error_msg<E>(self, message: E) -> Error
    where
        E: Debug + Display + Send + Sync + 'static,
    {
        Error::from(
            OutputError::new(self.command, Box::new(self.output)).with_message(Box::new(message)),
        )
    }

    pub(crate) fn maybe_error_msg<E>(self, message: Option<E>) -> Error
    where
        E: Debug + Display + Send + Sync + 'static,
    {
        let ret = OutputError::new(self.command, Box::new(self.output));
        Error::from(match message {
            Some(message) => ret.with_message(Box::new(message)),
            None => ret,
        })
    }
}
