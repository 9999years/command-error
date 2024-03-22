use std::borrow::Cow;
use std::fmt::Display;
use std::process::Command;

use crate::CommandDisplay;

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
