use std::borrow::Cow;
use std::fmt::Display;

#[cfg(doc)]
use std::process::Command;

use dyn_clone::DynClone;

#[cfg(doc)]
use crate::Utf8ProgramAndArgs;

/// A [`Command`] that can be [`Display`]ed.
///
/// The command's program and arguments are provided as strings, which may contain � U+FFFD
/// REPLACEMENT CHARACTER if the program or arguments cannot be decoded as UTF-8.
///
/// The [`Display`] implementation in [`Utf8ProgramAndArgs`] additionally performs shell quoting on
/// the command's program and args.
pub trait CommandDisplay: Display + DynClone {
    /// The command's program name, decoded as UTF-8.
    ///
    /// ```
    /// # use std::process::Command;
    /// # use command_error::Utf8ProgramAndArgs;
    /// # use command_error::CommandDisplay;
    /// let command = Command::new("echo");
    /// let displayed: Utf8ProgramAndArgs = (&command).into();
    /// assert_eq!(
    ///     displayed.program(),
    ///     "echo",
    /// );
    /// ```
    fn program(&self) -> Cow<'_, str>;

    /// The command's program name, shell-quoted.
    ///
    /// ```
    /// # use std::process::Command;
    /// # use command_error::Utf8ProgramAndArgs;
    /// # use command_error::CommandDisplay;
    /// let command = Command::new("ooga booga");
    /// let displayed: Utf8ProgramAndArgs = (&command).into();
    /// assert_eq!(
    ///     displayed.program_quoted(),
    ///     "'ooga booga'",
    /// );
    /// ```
    fn program_quoted(&self) -> Cow<'_, str> {
        Cow::Owned(shell_words::quote(&self.program()).into_owned())
    }

    /// The command's arguments, decoded as UTF-8.
    ///
    /// ```
    /// # use std::process::Command;
    /// # use command_error::Utf8ProgramAndArgs;
    /// # use command_error::CommandDisplay;
    /// let mut command = Command::new("echo");
    /// command.arg("puppy doggy");
    /// let displayed: Utf8ProgramAndArgs = (&command).into();
    /// assert_eq!(
    ///     displayed.args().collect::<Vec<_>>(),
    ///     vec!["puppy doggy"],
    /// );
    /// ```
    fn args(&self) -> Box<dyn Iterator<Item = Cow<'_, str>> + '_>;
}

impl Clone for Box<dyn CommandDisplay + Send + Sync> {
    fn clone(&self) -> Self {
        dyn_clone::clone_box(&**self)
    }
}
