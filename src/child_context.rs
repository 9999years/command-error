use std::borrow::Borrow;
use std::fmt::Debug;

#[cfg(doc)]
use std::process::Child;
#[cfg(doc)]
use std::process::Command;

#[cfg(doc)]
use crate::ChildExt;
use crate::CommandDisplay;
#[cfg(doc)]
use crate::OutputContext;

/// A [`Child`] process combined with context about the [`Command`] that produced it.
///
/// The context information stored in this type is used to produce diagnostics in [`ChildExt`].
///
/// See: [`OutputContext`].
pub struct ChildContext<C> {
    pub(crate) child: C,
    pub(crate) command: Box<dyn CommandDisplay + Send + Sync>,
}

impl<C> ChildContext<C> {
    /// Get the child process.
    pub fn into_child(self) -> C {
        self.child
    }

    /// Get a reference to the child process.
    pub fn child(&self) -> &C {
        &self.child
    }

    /// Get a mutable reference to the child process.
    pub fn child_mut(&mut self) -> &mut C {
        &mut self.child
    }

    /// Get a reference to the command which produced this child process.
    pub fn command(&self) -> &(dyn CommandDisplay + Send + Sync) {
        self.command.borrow()
    }
}

impl<C> Debug for ChildContext<C>
where
    C: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ChildContext")
            .field("child", &self.child)
            .field("command", &self.command.to_string())
            .finish()
    }
}
