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

/// Context around [`Command`] [`Output`].
///
/// This contains additional information about the command that was run (via a [`CommandDisplay`]
/// object) and can be used to construct error messages (for use in methods like
/// [`CommandExt::output_checked_as`]).
pub struct OutputContext<O> {
    pub(crate) output: O,
    pub(crate) command: Box<dyn CommandDisplay>,
}

impl<O> OutputContext<O>
where
    O: OutputLike + 'static,
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
    pub fn command(&self) -> &dyn CommandDisplay {
        self.command.borrow()
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
        E: Debug + Display + 'static,
    {
        Error::from(
            OutputError::new(self.command, Box::new(self.output))
                .with_user_error(Some(Box::new(message))),
        )
    }

    pub(crate) fn maybe_error_msg<E>(self, message: Option<E>) -> Error
    where
        E: Debug + Display + 'static,
    {
        let ret = OutputError::new(self.command, Box::new(self.output));
        Error::from(match message {
            Some(message) => ret.with_user_error(Some(Box::new(message))),
            None => ret,
        })
    }
}
