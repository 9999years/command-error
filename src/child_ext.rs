use std::borrow::Borrow;
use std::fmt::Debug;
use std::fmt::Display;
use std::process::Child;
use std::process::ExitStatus;
use std::process::Output;

use utf8_command::Utf8Output;

use crate::ChildContext;
#[cfg(doc)]
use crate::CommandExt;

use crate::Error;
use crate::ExecError;
use crate::OutputContext;
use crate::OutputConversionError;
use crate::OutputLike;
use crate::TryWaitContext;
use crate::WaitError;

/// Checked methods for [`Child`] processes.
///
/// This trait is largely the same as [`CommandExt`], with the difference that the
/// [`ChildExt::output_checked`] methods take `self` as an owned parameter and the
/// [`CommandExt::output_checked`] methods take `self` as a mutable reference.
///
/// Additionally, methods that return an [`ExitStatus`] are named
/// [`wait_checked`][`ChildExt::wait_checked`] instead of
/// [`status_checked`][`CommandExt::status_checked`], to match the method names on [`Child`].
pub trait ChildExt: Sized {
    /// The error type returned from methods on this trait.
    type Error: From<Error>;

    /// Wait for the process to complete, capturing its output. `succeeded` is called and returned
    /// to determine if the command succeeded.
    ///
    /// See [`CommandExt::output_checked_as`] for more information.
    #[track_caller]
    fn output_checked_as<O, R, E>(
        self,
        succeeded: impl Fn(OutputContext<O>) -> Result<R, E>,
    ) -> Result<R, E>
    where
        O: Debug,
        O: OutputLike,
        O: 'static,
        O: TryFrom<Output>,
        <O as TryFrom<Output>>::Error: Display,
        E: From<Self::Error>;

    /// Wait for the process to complete, capturing its output. `succeeded` is called and used to
    /// determine if the command succeeded and (optionally) to add an additional message to the error returned.
    ///
    /// See [`CommandExt::output_checked_with`] and [`Child::wait_with_output`] for more information.
    #[track_caller]
    fn output_checked_with<O, E>(
        self,
        succeeded: impl Fn(&O) -> Result<(), Option<E>>,
    ) -> Result<O, Self::Error>
    where
        O: Debug,
        O: OutputLike,
        O: TryFrom<Output>,
        <O as TryFrom<Output>>::Error: Display,
        O: 'static,
        E: Debug,
        E: Display,
        E: 'static,
    {
        self.output_checked_as(|context| match succeeded(context.output()) {
            Ok(()) => Ok(context.into_output()),
            Err(user_error) => Err(context.maybe_error_msg(user_error).into()),
        })
    }

    /// Wait for the process to complete, capturing its output. If the command exits with a
    /// non-zero exit code, an error is raised.
    ///
    /// See [`CommandExt::output_checked`] and [`Child::wait_with_output`] for more information.
    #[track_caller]
    fn output_checked(self) -> Result<Output, Self::Error> {
        self.output_checked_with(|output: &Output| {
            if output.status.success() {
                Ok(())
            } else {
                Err(None::<String>)
            }
        })
    }

    /// Wait for the process to exit, capturing its output and decoding it as UTF-8. If the command
    /// exits with a non-zero exit code, an error is raised.
    ///
    /// See [`CommandExt::output_checked_utf8`] and [`Child::wait_with_output`] for more information.
    #[track_caller]
    fn output_checked_utf8(self) -> Result<Utf8Output, Self::Error> {
        self.output_checked_with_utf8(|output| {
            if output.status.success() {
                Ok(())
            } else {
                Err(None::<String>)
            }
        })
    }

    /// Wait for the process to exit, capturing its output and decoding it as UTF-8. `succeeded` is
    /// called and used to determine if the command succeeded and (optionally) to add an additional
    /// message to the error returned.
    ///
    /// See [`CommandExt::output_checked_with_utf8`] and [`Child::wait_with_output`] for more information.
    #[track_caller]
    fn output_checked_with_utf8<E>(
        self,
        succeeded: impl Fn(&Utf8Output) -> Result<(), Option<E>>,
    ) -> Result<Utf8Output, Self::Error>
    where
        E: Display,
        E: Debug,
        E: 'static,
    {
        self.output_checked_with(succeeded)
    }

    /// Check if the process has exited.
    ///
    /// The `succeeded` closure is called and returned to determine the result.
    ///
    /// Errors while attempting to retrieve the process's exit status are returned as
    /// [`WaitError`]s.
    ///
    /// See [`Child::try_wait`] for more information.
    #[track_caller]
    fn try_wait_checked_as<R, E>(
        &mut self,
        succeeded: impl Fn(TryWaitContext) -> Result<R, E>,
    ) -> Result<R, E>
    where
        E: From<Self::Error>;

    /// Check if the process has exited and, if it failed, return an error.
    ///
    /// Errors while attempting to retrieve the process's exit status are transformed into
    /// [`WaitError`]s.
    ///
    /// See [`Child::try_wait`] for more information.
    #[track_caller]
    fn try_wait_checked(&mut self) -> Result<Option<ExitStatus>, Self::Error> {
        self.try_wait_checked_as(|context| match context.into_output_context() {
            Some(context) => {
                if context.status().success() {
                    Ok(Some(context.status()))
                } else {
                    Err(context.error().into())
                }
            }
            None => Ok(None),
        })
    }

    /// Wait for the process to exit. `succeeded` is called and returned to determine
    /// if the command succeeded.
    ///
    /// See [`CommandExt::status_checked_as`] and [`Child::wait`] for more information.
    #[track_caller]
    fn wait_checked_as<R, E>(
        &mut self,
        succeeded: impl Fn(OutputContext<ExitStatus>) -> Result<R, E>,
    ) -> Result<R, E>
    where
        E: From<Self::Error>;

    /// Wait for the process to exit. `succeeded` is called and used to determine
    /// if the command succeeded and (optionally) to add an additional message to the error
    /// returned.
    ///
    /// See [`CommandExt::status_checked_with`] and [`Child::wait`] for more information.
    #[track_caller]
    fn wait_checked_with<E>(
        &mut self,
        succeeded: impl Fn(ExitStatus) -> Result<(), Option<E>>,
    ) -> Result<ExitStatus, Self::Error>
    where
        E: Debug,
        E: Display,
        E: 'static,
    {
        self.wait_checked_as(|context| match succeeded(context.status()) {
            Ok(()) => Ok(context.status()),
            Err(user_error) => Err(context.maybe_error_msg(user_error).into()),
        })
    }

    /// Wait for the process to exit. If the command exits with a non-zero status
    /// code, an error is raised containing information about the command that was run.
    ///
    /// See [`CommandExt::status_checked`] and [`Child::wait`] for more information.
    #[track_caller]
    fn wait_checked(&mut self) -> Result<ExitStatus, Self::Error> {
        self.wait_checked_with(|status| {
            if status.success() {
                Ok(())
            } else {
                Err(None::<String>)
            }
        })
    }

    /// Log the command that will be run.
    ///
    /// With the `tracing` feature enabled, this will emit a debug-level log with message
    /// `Executing command` and a `command` field containing the displayed command (by default,
    /// shell-quoted).
    fn log(&self) -> Result<(), Self::Error>;
}

impl ChildExt for ChildContext<Child> {
    type Error = Error;

    fn output_checked_as<O, R, E>(
        self,
        succeeded: impl Fn(OutputContext<O>) -> Result<R, E>,
    ) -> Result<R, E>
    where
        O: Debug,
        O: OutputLike,
        O: 'static,
        O: TryFrom<Output>,
        <O as TryFrom<Output>>::Error: Display,
        E: From<Self::Error>,
    {
        self.log()?;
        let command = dyn_clone::clone_box(self.command.borrow());
        match self.child.wait_with_output() {
            Ok(output) => match output.try_into() {
                Ok(output) => succeeded(OutputContext { output, command }),
                Err(error) => Err(Error::from(OutputConversionError {
                    command,
                    inner: Box::new(error),
                })
                .into()),
            },
            Err(inner) => Err(Error::from(ExecError { command, inner }).into()),
        }
    }

    fn try_wait_checked_as<R, E>(
        &mut self,
        succeeded: impl Fn(TryWaitContext) -> Result<R, E>,
    ) -> Result<R, E>
    where
        E: From<Self::Error>,
    {
        let command = dyn_clone::clone_box(self.command.borrow());
        match self.child.try_wait() {
            Ok(status) => succeeded(TryWaitContext { status, command }),
            Err(inner) => Err(Error::from(WaitError { inner, command }).into()),
        }
    }

    fn wait_checked_as<R, E>(
        &mut self,
        succeeded: impl Fn(OutputContext<ExitStatus>) -> Result<R, E>,
    ) -> Result<R, E>
    where
        E: From<Self::Error>,
    {
        self.log()?;
        let command = dyn_clone::clone_box(self.command.borrow());
        match self.child.wait() {
            Ok(status) => succeeded(OutputContext {
                output: status,
                command,
            }),
            Err(inner) => Err(Error::from(ExecError { command, inner }).into()),
        }
    }

    fn log(&self) -> Result<(), Self::Error> {
        #[cfg(feature = "tracing")]
        {
            tracing::debug!(command = %self.command, "Executing command");
        }
        Ok(())
    }
}
