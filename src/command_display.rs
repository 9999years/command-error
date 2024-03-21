use std::borrow::Cow;
use std::fmt::Display;
use std::process::Command;

/// A [`Command`] that can be [`Display`]ed.
///
/// The command's program and arguments are provided as strings, which may contain � U+FFFD
/// REPLACEMENT CHARACTER if the program or arguments cannot be decoded as UTF-8.
///
/// The [`Display`] implementation in [`Utf8ProgramAndArgs`] additionally performs shell quoting on
/// the command's program and args.
pub trait CommandDisplay: Display {
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

/// A program name and arguments stored as UTF-8 [`String`]s.
///
/// The program name and arguments are shell-quoted when [`Display`]ed, so that spaces are escaped
/// and the displayed command can generally be pasted directly into a shell.
///
/// ```
/// # use std::process::Command;
/// # use command_error::Utf8ProgramAndArgs;
/// # use command_error::CommandDisplay;
/// let mut command = Command::new("echo");
/// command.arg("puppy doggy");
/// let displayed: Utf8ProgramAndArgs = (&command).into();
/// assert_eq!(
///     displayed.to_string(),
///     "echo 'puppy doggy'"
/// );
/// ```
#[derive(Debug, Clone)]
pub struct Utf8ProgramAndArgs {
    program: String,
    args: Vec<String>,
}

impl Display for Utf8ProgramAndArgs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", shell_words::quote(&self.program))?;
        if !self.args.is_empty() {
            write!(f, " {}", shell_words::join(&self.args))?;
        }
        Ok(())
    }
}

impl CommandDisplay for Utf8ProgramAndArgs {
    fn program(&self) -> std::borrow::Cow<'_, str> {
        Cow::Borrowed(&self.program)
    }

    fn program_quoted(&self) -> Cow<'_, str> {
        shell_words::quote(&self.program)
    }

    fn args(&self) -> Box<(dyn Iterator<Item = Cow<'_, str>> + '_)> {
        Box::new(self.args.iter().map(|arg| Cow::Borrowed(arg.as_str())))
    }
}

impl<'a> From<&'a Command> for Utf8ProgramAndArgs {
    fn from(command: &'a Command) -> Self {
        Utf8ProgramAndArgs {
            program: command.get_program().to_string_lossy().into_owned(),
            args: command
                .get_args()
                .map(|arg| arg.to_string_lossy().into_owned())
                .collect(),
        }
    }
}
