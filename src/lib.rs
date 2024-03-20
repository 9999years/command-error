//! Utilities for running commands and providing user-friendly error messages.

use std::fmt::Debug;
use std::fmt::Display;
use std::process::ExitStatus;
use std::process::{Command, Output as StdOutput};

#[cfg(feature = "utf8-command")]
use utf8_command::Utf8Output;

mod output;
pub use output::Output;

mod output_like;
pub use output_like::OutputLike;

mod exec_error;
pub use exec_error::ExecError;

mod output_error;
pub use output_error::OutputError;

mod output_conversion_error;
pub use output_conversion_error::OutputConversionError;

mod error;
pub use error::Error;

mod command_display;
pub use command_display::CommandDisplay;
pub use command_display::Utf8ProgramAndArgs;

mod debug_display;
pub(crate) use debug_display::DebugDisplay;

/// Extension trait for [`Command`], adding helpers to gather output while checking the exit
/// status.
pub trait CommandExt {
    type Error: From<Error>;
    type Displayed: for<'a> From<&'a Self> + CommandDisplay;

    /// Like `output_checked_with`, but the closure's return value is used as the function's return
    /// value.
    ///
    /// Useful to apply constraints to the output.
    #[track_caller]
    fn output_checked_as<O, R, E>(
        &mut self,
        succeeded: impl Fn(Output<O>) -> Result<R, E>,
    ) -> Result<R, E>
    where
        O: Debug,
        O: OutputLike,
        O: 'static,
        O: TryFrom<StdOutput>,
        <O as TryFrom<StdOutput>>::Error: Display,
        E: From<Self::Error>;

    /// Like [`output_checked`] but a closure determines if the command failed instead of
    /// [`ExitStatus::success`].
    #[track_caller]
    fn output_checked_with<O, E>(
        &mut self,
        succeeded: impl Fn(&O) -> Result<(), Option<E>>,
    ) -> Result<O, Self::Error>
    where
        O: Debug,
        O: OutputLike,
        O: TryFrom<StdOutput>,
        <O as TryFrom<StdOutput>>::Error: Display,
        O: 'static,
        E: Debug,
        E: Display,
        E: 'static,
    {
        self.output_checked_as(|output| match succeeded(output.output()) {
            Ok(()) => Ok(output.into_output()),
            Err(user_error) => Err(output.maybe_error_msg(user_error).into()),
        })
    }

    /// Like [`Command::output`], but checks the exit status and provides nice error messages.
    #[track_caller]
    fn output_checked(&mut self) -> Result<StdOutput, Self::Error> {
        self.output_checked_with(|output: &StdOutput| {
            if output.status.success() {
                Ok(())
            } else {
                Err(None::<String>)
            }
        })
    }

    /// Like [`output_checked`], but also decodes Stdout and Stderr as UTF-8.
    #[cfg(feature = "utf8-command")]
    #[track_caller]
    fn output_checked_utf8(&mut self) -> Result<Utf8Output, Self::Error> {
        self.output_checked_with_utf8(|output| {
            if output.status.success() {
                Ok(())
            } else {
                Err(None::<String>)
            }
        })
    }
    /// Like [`output_checked_with`], but also decodes Stdout and Stderr as UTF-8.
    #[cfg(feature = "utf8-command")]
    #[track_caller]
    fn output_checked_with_utf8<E>(
        &mut self,
        succeeded: impl Fn(&Utf8Output) -> Result<(), Option<E>>,
    ) -> Result<Utf8Output, Self::Error>
    where
        E: Display,
        E: Debug,
        E: 'static,
    {
        self.output_checked_with(succeeded)
    }

    /// Like [`Command::status`], but gives a nice error message if the status is unsuccessful.
    #[track_caller]
    fn status_checked_as<R, E>(
        &mut self,
        succeeded: impl Fn(Output<ExitStatus>) -> Result<R, E>,
    ) -> Result<R, E>
    where
        E: From<Self::Error>;

    /// Like [`Command::status`], but gives a nice error message if the status is unsuccessful.
    #[track_caller]
    fn status_checked_with<E>(
        &mut self,
        succeeded: impl Fn(ExitStatus) -> Result<(), Option<E>>,
    ) -> Result<ExitStatus, Self::Error>
    where
        E: Debug,
        E: Display,
        E: 'static,
    {
        self.status_checked_as(|status| match succeeded(status.status()) {
            Ok(()) => Ok(status.status()),
            Err(user_error) => Err(status.maybe_error_msg(user_error).into()),
        })
    }

    /// Like [`Command::status`], but gives a nice error message if the status is unsuccessful.
    #[track_caller]
    fn status_checked(&mut self) -> Result<ExitStatus, Self::Error> {
        self.status_checked_with(|status| {
            if status.success() {
                Ok(())
            } else {
                Err(None::<String>)
            }
        })
    }

    /// Log the command that will be run.
    fn log(&self) -> Result<(), Self::Error>;
}

impl CommandExt for Command {
    type Error = Error;
    type Displayed = Utf8ProgramAndArgs;

    fn log(&self) -> Result<(), Self::Error> {
        #[cfg(feature = "tracing")]
        {
            let command: Utf8ProgramAndArgs = self.into();
            tracing::debug!(%command, "Executing command");
        }
        Ok(())
    }

    fn output_checked_as<O, R, E>(
        &mut self,
        succeeded: impl Fn(Output<O>) -> Result<R, E>,
    ) -> Result<R, E>
    where
        O: Debug,
        O: OutputLike,
        O: 'static,
        O: TryFrom<StdOutput>,
        <O as TryFrom<StdOutput>>::Error: Display,
        E: From<Self::Error>,
    {
        let (output, displayed): (O, Self::Displayed) = get_output_as(self)?;
        succeeded(Output {
            output,
            command: Box::new(displayed),
        })
    }

    fn status_checked_as<R, E>(
        &mut self,
        succeeded: impl Fn(Output<ExitStatus>) -> Result<R, E>,
    ) -> Result<R, E>
    where
        E: From<Self::Error>,
    {
        self.log()?;
        let displayed: Utf8ProgramAndArgs = (&*self).into();
        let displayed = Box::new(displayed);
        match self.status() {
            Ok(status) => succeeded(Output {
                output: status,
                command: displayed,
            }),
            Err(inner) => Err(Error::from(ExecError {
                command: displayed,
                inner,
            })
            .into()),
        }
    }
}

fn get_output_as<O, D>(cmd: &mut Command) -> Result<(O, D), Error>
where
    O: TryFrom<StdOutput>,
    O: Debug + OutputLike + 'static,
    <O as TryFrom<StdOutput>>::Error: Display,
    D: CommandDisplay + for<'a> From<&'a Command> + 'static,
{
    cmd.log()?;
    let displayed: D = (&*cmd).into();
    match cmd.output() {
        Ok(output) => match output.try_into() {
            Ok(output) => Ok((output, displayed)),
            Err(error) => Err(Error::from(OutputConversionError {
                command: Box::new(displayed),
                inner: Box::new(error),
            })),
        },
        Err(inner) => Err(Error::from(ExecError {
            command: Box::new(displayed),
            inner,
        })),
    }
}
