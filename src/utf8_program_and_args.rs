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
///
/// let mut command = Command::new("echo");
/// command.arg("doggy")
///     .current_dir("/puppy")
///     .env("COLOR", "GOLDEN")
///     .env_remove("STINKY");
/// let displayed: Utf8ProgramAndArgs = (&command).into();
/// assert_eq!(
///     displayed.to_string(),
///     "cd /puppy && COLOR=GOLDEN STINKY= echo doggy"
/// );
/// ```
#[derive(Debug, Clone)]
pub struct Utf8ProgramAndArgs {
    current_dir: Option<String>,
    envs: Vec<(String, Option<String>)>,
    program: String,
    args: Vec<String>,
}

impl Display for Utf8ProgramAndArgs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(current_dir) = &self.current_dir {
            write!(f, "cd {} && ", shell_words::quote(current_dir))?;
        }

        for (key, value) in self.envs.iter() {
            // TODO: Should I care about spaces in environment variable names???
            write!(
                f,
                "{key}={} ",
                value
                    .as_deref()
                    .map(|value| shell_words::quote(value))
                    .unwrap_or_default()
            )?;
        }

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
            current_dir: command
                .get_current_dir()
                .map(|path| path.to_string_lossy().into_owned()),
            envs: command
                .get_envs()
                .map(|(key, value)| {
                    (
                        key.to_string_lossy().into_owned(),
                        value.map(|value| value.to_string_lossy().into_owned()),
                    )
                })
                .collect(),
            program: command.get_program().to_string_lossy().into_owned(),
            args: command
                .get_args()
                .map(|arg| arg.to_string_lossy().into_owned())
                .collect(),
        }
    }
}
