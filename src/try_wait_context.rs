use std::borrow::Borrow;
use std::fmt::Debug;
use std::process::ExitStatus;

#[cfg(doc)]
use std::process::Child;
#[cfg(doc)]
use std::process::Command;

use crate::CommandDisplay;
use crate::OutputContext;

/// An optional [`ExitStatus`] combined with context about the [`Command`] that produced it.
///
/// See also: [`OutputContext`].
pub struct TryWaitContext {
    pub(crate) status: Option<ExitStatus>,
    pub(crate) command: Box<dyn CommandDisplay + Send + Sync>,
}

impl TryWaitContext {
    /// Get the result of the [`Child::try_wait`] call.
    pub fn status(&self) -> Option<ExitStatus> {
        self.status
    }

    /// Get a reference to the command contained in this context object, for use in error messages
    /// or diagnostics.
    pub fn command(&self) -> &(dyn CommandDisplay + Send + Sync) {
        self.command.borrow()
    }

    /// Get the command contained in this context object, for use in error messages or diagnostics.
    pub fn into_command(self) -> Box<(dyn CommandDisplay + Send + Sync)> {
        self.command
    }

    /// If the [`ExitStatus`] is present, get an [`OutputContext`] for constructing error messages.
    pub fn into_output_context(self) -> Option<OutputContext<ExitStatus>> {
        self.status
            .map(|status| OutputContext::new(status, self.command))
    }
}

impl Debug for TryWaitContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TryWaitContext")
            .field("status", &self.status)
            .field("command", &self.command.to_string())
            .finish()
    }
}
