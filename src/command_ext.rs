use std::fmt::Debug;
use std::fmt::Display;
use std::process::ExitStatus;
use std::process::{Command, Output};

use utf8_command::Utf8Output;

use crate::CommandDisplay;
use crate::Error;
use crate::ExecError;
use crate::OutputContext;
use crate::OutputConversionError;
use crate::OutputLike;
use crate::Utf8ProgramAndArgs;

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
    /// The error type returned from methods on this trait.
    type Error: From<Error>;

    /// Run a command, capturing its output. `succeeded` is called and returned to determine if the
    /// command succeeded.
    ///
    /// This is the most general [`CommandExt`] method, and gives the caller full control over
    /// success logic and the output and errors produced.
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

    /// Run a command, capturing its output. `succeeded` is called and used to determine if the
    /// command succeeded and (optionally) to add an additional message to the error returned.
    ///
    /// This method is best if you want to consider a command successful if it has a non-zero exit
    /// code, or if its output contains some special string. If you'd like to additionally produce
    /// output that can't be produced with [`TryFrom<Output>`] (such as to deserialize a data
    /// structure), [`CommandExt::output_checked_as`] provides full control over the produced
    /// result.
    ///
    /// ```
    /// # use indoc::indoc;
    /// # use std::process::Command;
    /// # use std::process::Output;
    /// # use command_error::CommandExt;
    /// let output = Command::new("sh")
    ///     .args(["-c", "echo puppy && exit 2"])
    ///     .output_checked_with(|output: &Output| {
    ///         if let Some(2) = output.status.code() {
    ///             Ok(())
    ///         } else {
    ///             // Don't add any additional context to the error message:
    ///             Err(None::<String>)
    ///         }
    ///     })
    ///     .unwrap();
    ///
    /// assert_eq!(
    ///     output.status.code(),
    ///     Some(2),
    /// );
    /// ```
    ///
    /// Note that due to the generic error parameter, you'll need to annotate [`None`] return
    /// values with a [`Display`]able type — try [`String`] or any [`std::error::Error`] type in
    /// scope.
    ///
    /// [`Command::output_checked_with`] can also be used to convert the output to any type that
    /// implements [`TryFrom<Output>`] before running `succeeded`:
    ///
    /// ```
    /// # use indoc::indoc;
    /// # use std::process::Command;
    /// # use command_error::CommandExt;
    /// # use utf8_command::Utf8Output;
    /// let err = Command::new("sh")
    ///     .args(["-c", "echo kitty && kill -9 \"$$\""])
    ///     .output_checked_with(|output: &Utf8Output| {
    ///         if output.status.success() && output.stdout.trim() == "puppy" {
    ///             Ok(())
    ///         } else {
    ///             Err(Some("didn't find any puppy!"))
    ///         }
    ///     })
    ///     .unwrap_err();
    ///
    /// assert_eq!(
    ///     err.to_string(),
    ///     indoc!(
    ///         r#"`sh` failed: didn't find any puppy!
    ///         signal: 9 (SIGKILL)
    ///         Command failed: `sh -c 'echo kitty && kill -9 "$$"'`
    ///         Stdout:
    ///           kitty"#
    ///     )
    /// );
    /// ```
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

    /// Run a command, capturing its output. If the command exits with a non-zero exit code, an
    /// error is raised. Error messages are detailed and contain information about the command that
    /// was run and its output:
    ///
    /// ```
    /// # use pretty_assertions::assert_eq;
    /// # use indoc::indoc;
    /// # use std::process::Command;
    /// # use command_error::CommandExt;
    /// let err = Command::new("ooby-gooby")
    ///     .output_checked()
    ///     .unwrap_err();
    ///
    /// assert_eq!(
    ///     err.to_string(),
    ///     "Failed to execute `ooby-gooby`: No such file or directory (os error 2)"
    /// );
    ///
    /// let err = Command::new("sh")
    ///     .args(["-c", "echo puppy && exit 1"])
    ///     .output_checked()
    ///     .unwrap_err();
    /// assert_eq!(
    ///     err.to_string(),
    ///     indoc!(
    ///         "`sh` failed: exit status: 1
    ///         Command failed: `sh -c 'echo puppy && exit 1'`
    ///         Stdout:
    ///           puppy"
    ///     )
    /// );
    /// ```
    ///
    /// If the command fails, output will be decoded as UTF-8 for display in error messages, but
    /// otherwise no output decoding is performed. To decode output as UTF-8, use
    /// [`CommandExt::output_checked_utf8`]. To decode as other formats, use
    /// [`CommandExt::output_checked_with`].
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

    /// Run a command, capturing its output and decoding it as UTF-8. If the command exits with a
    /// non-zero exit code or if its output contains invalid UTF-8, an error is raised.
    ///
    /// See [`CommandExt::output_checked`] for more information.
    ///
    /// ```
    /// # use pretty_assertions::assert_eq;
    /// # use indoc::indoc;
    /// # use std::process::Command;
    /// # use std::process::ExitStatus;
    /// # use command_error::CommandExt;
    /// # use utf8_command::Utf8Output;
    /// let output = Command::new("echo")
    ///     .arg("puppy")
    ///     .output_checked_utf8()
    ///     .unwrap();
    ///
    /// assert_eq!(
    ///     output,
    ///     Utf8Output {
    ///         status: ExitStatus::default(),
    ///         stdout: "puppy\n".into(),
    ///         stderr: "".into(),
    ///     },
    /// );
    /// ```
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
    /// Run a command, capturing its output and decoding it as UTF-8. `succeeded` is called and
    /// used to determine if the command succeeded and (optionally) to add an additional message to
    /// the error returned.
    ///
    /// See [`CommandExt::output_checked_with`] for more information.
    ///
    /// ```
    /// # use pretty_assertions::assert_eq;
    /// # use indoc::indoc;
    /// # use std::process::Command;
    /// # use std::process::ExitStatus;
    /// # use command_error::CommandExt;
    /// # use utf8_command::Utf8Output;
    /// let output = Command::new("sh")
    ///     .args(["-c", "echo puppy; exit 1"])
    ///     .output_checked_with_utf8(|output| {
    ///         if output.stdout.contains("puppy") {
    ///             Ok(())
    ///         } else {
    ///             Err(None::<String>)
    ///         }
    ///     })
    ///     .unwrap();
    ///
    /// assert_eq!(output.stdout, "puppy\n");
    /// assert_eq!(output.status.code(), Some(1));
    /// ```
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

    /// Run a command without capturing its output. `succeeded` is called and returned to determine
    /// if the command succeeded.
    ///
    /// This gives the caller full control over success logic and the output and errors produced.
    ///
    /// ```
    /// # use pretty_assertions::assert_eq;
    /// # use indoc::indoc;
    /// # use std::process::Command;
    /// # use std::process::ExitStatus;
    /// # use command_error::CommandExt;
    /// # use command_error::OutputContext;
    /// let succeeded = |context: OutputContext<ExitStatus>| {
    ///     match context.status().code() {
    ///         Some(code) => Ok(code),
    ///         None => Err(context.error_msg("no exit code")),
    ///     }
    /// };
    ///
    /// let code = Command::new("true")
    ///     .status_checked_as(succeeded)
    ///     .unwrap();
    /// assert_eq!(code, 0);
    ///
    /// let err = Command::new("sh")
    ///     .args(["-c", "kill \"$$\""])
    ///     .status_checked_as(succeeded)
    ///     .unwrap_err();
    /// assert_eq!(
    ///     err.to_string(),
    ///     indoc!(
    ///         r#"`sh` failed: no exit code
    ///         signal: 15 (SIGTERM)
    ///         Command failed: `sh -c 'kill "$$"'`"#
    ///     )
    /// );
    /// ```
    ///
    /// To error on non-zero exit codes, use [`CommandExt::status_checked`].
    #[track_caller]
    fn status_checked_as<R, E>(
        &mut self,
        succeeded: impl Fn(OutputContext<ExitStatus>) -> Result<R, E>,
    ) -> Result<R, E>
    where
        E: From<Self::Error>;

    /// Run a command without capturing its output. `succeeded` is called and used to determine
    /// if the command succeeded and (optionally) to add an additional message to the error
    /// returned.
    ///
    /// ```
    /// # use pretty_assertions::assert_eq;
    /// # use indoc::indoc;
    /// # use std::process::Command;
    /// # use std::process::ExitStatus;
    /// # use command_error::CommandExt;
    /// # use command_error::OutputContext;
    /// let status = Command::new("false")
    ///     .status_checked_with(|status| {
    ///         match status.code() {
    ///             // Exit codes 0 and 1 are OK.
    ///             Some(0) | Some(1) => Ok(()),
    ///             // Other exit codes are errors.
    ///             _ => Err(None::<String>)
    ///         }
    ///     })
    ///     .unwrap();
    /// assert_eq!(status.code(), Some(1));
    /// ```
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

    /// Run a command without capturing its output. If the command exits with a non-zero status
    /// code, an error is raised containing information about the command that was run:
    ///
    /// ```
    /// # use pretty_assertions::assert_eq;
    /// # use indoc::indoc;
    /// # use std::process::Command;
    /// # use std::process::ExitStatus;
    /// # use command_error::CommandExt;
    /// let err = Command::new("sh")
    ///     .args(["-c", "exit 1"])
    ///     .status_checked()
    ///     .unwrap_err();
    ///
    /// assert_eq!(
    ///     err.to_string(),
    ///     indoc!(
    ///         "`sh` failed: exit status: 1
    ///         Command failed: `sh -c 'exit 1'`"
    ///     )
    /// );
    /// ```
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
    ///
    /// With the `tracing` feature enabled, this will emit a debug-level log with message
    /// `Executing command` and a `command` field containing the displayed command (by default,
    /// shell-quoted).
    fn log(&self) -> Result<(), Self::Error>;
}

impl CommandExt for Command {
    type Error = Error;

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
        let (output, displayed): (O, Utf8ProgramAndArgs) = get_output_as(self)?;
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
