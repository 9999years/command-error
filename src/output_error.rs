use std::fmt::Debug;
use std::fmt::Display;

use crate::CommandDisplay;
use crate::DebugDisplay;
use crate::OutputLike;

#[cfg(doc)]
use crate::CommandExt;

#[cfg(doc)]
use crate::ExecError;

/// An error from a failed command, typically due to a non-zero exit status.
///
/// Produced by [`CommandExt`]. This indicates a command that failed, typically with a non-zero
/// exit code, rather than a command that failed to start (as in [`ExecError`]).
///
/// ```
/// # use pretty_assertions::assert_eq;
/// # use indoc::indoc;
/// # use std::process::Command;
/// # use std::process::Output;
/// # use std::process::ExitStatus;
/// # use command_error::Utf8ProgramAndArgs;
/// # use command_error::CommandDisplay;
/// # use command_error::OutputError;
/// let mut command = Command::new("sh");
/// command.args(["-c", "echo puppy doggy"]);
/// let displayed: Utf8ProgramAndArgs = (&command).into();
/// let error = OutputError::new(
///     Box::new(displayed),
///     Box::new(Output {
///         status: ExitStatus::default(),
///         stdout: "puppy doggy\n".as_bytes().to_vec(),
///         stderr: Vec::new(),
///     })
/// );
/// assert_eq!(
///     error.to_string(),
///     indoc!(
///         "`sh` failed: exit status: 0
///         Command failed: `sh -c 'echo puppy doggy'`
///         Stdout:
///           puppy doggy"
///     ),
/// );
/// assert_eq!(
///     error.with_message(Box::new("no kitties found!")).to_string(),
///     indoc!(
///         "`sh` failed: no kitties found!
///         exit status: 0
///         Command failed: `sh -c 'echo puppy doggy'`
///         Stdout:
///           puppy doggy"
///     )
/// );
/// ```
pub struct OutputError {
    /// The program and arguments that ran.
    pub(crate) command: Box<dyn CommandDisplay + Send + Sync>,
    /// The program's output and exit code.
    pub(crate) output: Box<dyn OutputLike + Send + Sync>,
    /// A user-defined error message.
    pub(crate) user_error: Option<Box<dyn DebugDisplay + Send + Sync>>,
}

impl OutputError {
    /// Construct a new [`OutputError`].
    pub fn new(
        command: Box<dyn CommandDisplay + Send + Sync>,
        output: Box<dyn OutputLike + Send + Sync>,
    ) -> Self {
        Self {
            command,
            output,
            user_error: None,
        }
    }

    /// Attach a user-defined message to this error.
    pub fn with_message(mut self, message: Box<dyn DebugDisplay + Send + Sync>) -> Self {
        self.user_error = Some(message);
        self
    }

    /// Remove a user-defined message from this error, if any.
    pub fn without_message(mut self) -> Self {
        self.user_error = None;
        self
    }
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
        write!(f, "`{}` failed: ", self.command.program_quoted())?;

        match &self.user_error {
            Some(user_error) => {
                // `nix` failed: output didn't contain a valid store path
                // exit status 0
                write!(f, "{user_error}\n{}", self.output.status())?;
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
            writeln!(f, "\nStdout:")?;
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
            writeln!(f, "\nStderr:")?;
            write_indented(f, stderr, INDENT)?;
        }
        Ok(())
    }
}

impl std::error::Error for OutputError {}

fn write_indented(f: &mut std::fmt::Formatter<'_>, text: &str, indent: &str) -> std::fmt::Result {
    let mut lines = text.lines();
    if let Some(line) = lines.next() {
        write!(f, "{indent}{line}")?;
        for line in lines {
            write!(f, "\n{indent}{line}")?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use static_assertions::assert_impl_all;

    assert_impl_all!(OutputError: Send, Sync);
}
