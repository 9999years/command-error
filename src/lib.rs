//! `command_error` provides the [`CommandExt`] trait, which runs a command and checks its exit
//! status:
//!
//! ```
//! # use indoc::indoc;
//! use std::process::Command;
//! use command_error::CommandExt;
//!
//! let err = Command::new("sh")
//!     .args(["-c", "echo puppy; false"])
//!     .output_checked_utf8()
//!     .unwrap_err();
//!
//! assert_eq!(
//!     err.to_string(),
//!     indoc!(
//!         "`sh` failed: exit status: 1
//!         Command failed: `sh -c 'echo puppy; false'`
//!         Stdout:
//!           puppy"
//!     )
//! );
//! ```
//!
//! Error messages are detailed and helpful. Additional methods are provided for overriding
//! the default success logic (for that weird tool that thinks `2` is a reasonable exit code) and
//! for transforming the output (for example, to parse command output as JSON while retaining
//! information about the command that produced the output in the error message).
//!
//! ## Enforcing use of [`CommandExt`]
//!
//! If you'd like to make sure that [`CommandExt`] methods are used instead of the plain
//! [`Command`] methods in your project, you can add a stanza like this to
//! [`clippy.toml`][clippy-config] at your project root:
//!
//! ```toml
//! [[disallowed-methods]]
//! path = "std::process::Command::output"
//! reason = "Use command_error::CommandExt::output_checked[_with][_utf8]"
//!
//! [[disallowed-methods]]
//! path = "std::process::Command::status"
//! reason = "Use command_error::CommandExt::status_checked[_with]"
//! ```
//!
//! [clippy-config]: https://doc.rust-lang.org/clippy/configuration.html

use std::fmt::Debug;
use std::fmt::Display;
use std::process::ExitStatus;
use std::process::{Command, Output};

#[cfg(feature = "utf8-command")]
use utf8_command::Utf8Output;

mod output;
pub use output::OutputContext;

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

/// Extension trait for [`Command`].
///
/// [`CommandExt`] methods check the exit status of the command (or perform user-supplied
/// validation logic) and produced detailed, helpful error messages when they fail:
///
/// ```
/// # use indoc::indoc;
/// use std::process::Command;
/// use command_error::CommandExt;
///
/// let err = Command::new("sh")
///     .args(["-c", "echo puppy; false"])
///     .output_checked_utf8()
///     .unwrap_err();
///
/// assert_eq!(
///     err.to_string(),
///     indoc!(
///         "`sh` failed: exit status: 1
///         Command failed: `sh -c 'echo puppy; false'`
///         Stdout:
///           puppy"
///     )
/// );
/// ```
///
/// With the `tracing` feature enabled, commands will be logged before they run.
///
/// # Method overview
///
/// | Method | Output decoding | Validation |
/// | ------ | --------------- | ---------- |
/// | [`output_checked`][CommandExt::output_checked`] | Bytes | Non-zero exit codes are errors |
/// | [`output_checked_utf8`][CommandExt::output_checked_utf8`] | UTF-8 | Non-zero exit codes are errors |
/// | [`output_checked_with`][CommandExt::output_checked_with`] | Arbitrary | Custom |
/// | [`output_checked_with_utf8`][CommandExt::output_checked_with_utf8`] | UTF-8 | Custom |
/// | [`output_checked_as`][CommandExt::output_checked_as`] | Arbitrary | Custom, with arbitrary error type |
/// | [`status_checked_as`][CommandExt::status_checked_as`] | None | Custom, with arbitrary error type |
/// | [`status_checked`][CommandExt::status_checked`] | None | Non-zero exit codes are errors |
pub trait CommandExt {
    type Error: From<Error>;
    type Displayed: for<'a> From<&'a Self> + CommandDisplay;

    /// Run a command, capturing its output. The given closure is used to determine if the command
    /// succeeded and to produce the method's output.
    ///
    /// ```
    /// # use indoc::indoc;
    /// # use std::process::Command;
    /// # use std::process::Output;
    /// # use command_error::CommandExt;
    /// # use command_error::OutputContext;
    /// # mod serde_json {
    /// #     /// Teehee!
    /// #     pub fn from_slice(_input: &[u8]) -> Result<Vec<String>, String> {
    /// #         Err("EOF while parsing a list at line 4 column 11".into())
    /// #     }
    /// # }
    /// let err = Command::new("cat")
    ///     .arg("tests/data/incomplete.json")
    ///     .output_checked_as(|context: OutputContext<Output>| {
    ///         serde_json::from_slice(&context.output().stdout)
    ///             .map_err(|err| context.error_msg(err))
    ///     })
    ///     .unwrap_err();
    ///
    /// assert_eq!(
    ///     err.to_string(),
    ///     indoc!(
    ///         r#"`cat` failed: EOF while parsing a list at line 4 column 11
    ///         exit status: 0
    ///         Command failed: `cat tests/data/incomplete.json`
    ///         Stdout:
    ///           [
    ///               "cuppy",
    ///               "dog",
    ///               "city","#
    ///     )
    /// );
    /// ```
    ///
    /// Note that the closure takes the output as raw bytes but the error message contains the
    /// output decoded as UTF-8. In this example, the decoding only happens in the error case, but
    /// if you request an [`OutputContext<Utf8Output>`], the decoded data will be reused for the
    /// error message.
    ///
    /// The [`OutputContext`] passed to the closure contains information about the command's
    /// [`Output`] (including its [`ExitStatus`]), the command that ran (the program name and its
    /// arguments), and methods for constructing detailed error messages (with or without
    /// additional context information).
    #[track_caller]
    fn output_checked_as<O, R, E>(
        &mut self,
        succeeded: impl Fn(OutputContext<O>) -> Result<R, E>,
    ) -> Result<R, E>
    where
        O: Debug,
        O: OutputLike,
        O: 'static,
        O: TryFrom<Output>,
        <O as TryFrom<Output>>::Error: Display,
        E: From<Self::Error>;

    /// Like [`CommandExt::output_checked`] but a closure determines if the command failed instead of
    /// [`ExitStatus::success`].
    #[track_caller]
    fn output_checked_with<O, E>(
        &mut self,
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
        self.output_checked_as(|output| match succeeded(output.output()) {
            Ok(()) => Ok(output.into_output()),
            Err(user_error) => Err(output.maybe_error_msg(user_error).into()),
        })
    }

    /// Like [`Command::output`], but checks the exit status and provides nice error messages.
    #[track_caller]
    fn output_checked(&mut self) -> Result<Output, Self::Error> {
        self.output_checked_with(|output: &Output| {
            if output.status.success() {
                Ok(())
            } else {
                Err(None::<String>)
            }
        })
    }

    /// Like [`CommandExt::output_checked`], but also decodes Stdout and Stderr as UTF-8.
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
    /// Like [`CommandExt::output_checked_with`], but also decodes Stdout and Stderr as UTF-8.
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
        succeeded: impl Fn(OutputContext<ExitStatus>) -> Result<R, E>,
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
        let (output, displayed): (O, Self::Displayed) = get_output_as(self)?;
        succeeded(OutputContext {
            output,
            command: Box::new(displayed),
        })
    }

    fn status_checked_as<R, E>(
        &mut self,
        succeeded: impl Fn(OutputContext<ExitStatus>) -> Result<R, E>,
    ) -> Result<R, E>
    where
        E: From<Self::Error>,
    {
        self.log()?;
        let displayed: Utf8ProgramAndArgs = (&*self).into();
        let displayed = Box::new(displayed);
        match self.status() {
            Ok(status) => succeeded(OutputContext {
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
    O: TryFrom<Output>,
    O: Debug + OutputLike + 'static,
    <O as TryFrom<Output>>::Error: Display,
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
