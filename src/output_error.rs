use std::fmt::Debug;
use std::fmt::Display;

use crate::CommandDisplay;
use crate::DebugDisplay;
use crate::OutputLike;

/// An error from a failed command. Produced by [`CommandExt`].
pub struct OutputError {
    /// The program and arguments that ran.
    pub(crate) command: Box<dyn CommandDisplay>,
    /// The program's output and exit code.
    pub(crate) output: Box<dyn OutputLike>,
    /// A user-defined error message.
    pub(crate) user_error: Option<Box<dyn DebugDisplay>>,
}

impl Debug for OutputError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OutputError")
            .field("program", &self.command.program())
            .field("status", &self.output.status())
            .field("stdout_utf8", &self.output.stdout())
            .field("stderr_utf8", &self.output.stderr())
            .field("user_error", &self.user_error)
            .finish()
    }
}

impl Display for OutputError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "`{}` failed: ", self.command.program())?;
        match &self.user_error {
            Some(user_error) => {
                // `nix` failed: output didn't contain a valid store path
                write!(f, "{user_error}")?;
            }
            None => {
                // `nix` failed: exit status: 1
                write!(f, "{}", self.output.status())?;
            }
        }

        // Command failed: `nix build .#default`
        write!(f, "\nCommand failed: `{}`", self.command,)?;

        const INDENT: &str = "  ";

        let stdout = self.output.stdout();
        let stdout = stdout.trim();
        if !stdout.is_empty() {
            writeln!(f, "Stdout:")?;
            write_indented(f, stdout, INDENT)?;
        }

        // Stdout:
        //   ...
        // Stderr:
        //   ...
        //   ...
        let stderr = self.output.stderr();
        let stderr = stderr.trim();
        if !stderr.is_empty() {
            writeln!(f, "Stderr:")?;
            write_indented(f, stderr, INDENT)?;
        }
        Ok(())
    }
}

impl std::error::Error for OutputError {}

impl OutputError {
    pub(crate) fn new(command: Box<dyn CommandDisplay>, output: Box<dyn OutputLike>) -> Self {
        Self {
            command,
            output,
            user_error: None,
        }
    }

    pub(crate) fn with_user_error(mut self, user_error: Option<Box<dyn DebugDisplay>>) -> Self {
        self.user_error = user_error;
        self
    }
}

fn write_indented(f: &mut std::fmt::Formatter<'_>, text: &str, indent: &str) -> std::fmt::Result {
    for line in text.lines() {
        write!(f, "{indent}{line}")?;
    }
    Ok(())
}