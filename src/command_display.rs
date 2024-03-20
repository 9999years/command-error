use std::borrow::Cow;
use std::fmt::Display;
use std::process::Command;

pub trait CommandDisplay: Display {
    fn program(&self) -> Cow<'_, str>;
    fn args(&self) -> Box<dyn Iterator<Item = Cow<'_, str>> + '_>;
}

#[derive(Debug, Clone)]
pub struct Utf8ProgramAndArgs {
    program: String,
    args: Vec<String>,
}

#[cfg(feature = "shell-words")]
impl Display for Utf8ProgramAndArgs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", shell_words::quote(&self.program))?;
        if !self.args.is_empty() {
            write!(f, " {}", shell_words::join(&self.args))?;
        }
        Ok(())
    }
}

#[cfg(not(feature = "shell-words"))]
impl Display for Utf8ProgramAndArgs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", &self.program)?;
        for arg in &self.args {
            write!(f, " {arg:?}")?;
        }
        Ok(())
    }
}

impl CommandDisplay for Utf8ProgramAndArgs {
    fn program(&self) -> std::borrow::Cow<'_, str> {
        Cow::Borrowed(&self.program)
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
