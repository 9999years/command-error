use std::borrow::Cow;
use std::process::ExitStatus;
use std::process::Output;

#[cfg(feature = "utf8-command")]
use utf8_command::Utf8Output;

/// A command output type.
pub trait OutputLike {
    /// The command's exit status.
    fn status(&self) -> ExitStatus;

    /// The command's stdout, decoded to UTF-8 on a best-effort basis.
    fn stdout(&self) -> Cow<'_, str>;

    /// The command's stderr, decoded to UTF-8 on a best-effort basis.
    fn stderr(&self) -> Cow<'_, str>;
}

/// A trivial implementation with empty output.
impl OutputLike for ExitStatus {
    fn status(&self) -> ExitStatus {
        *self
    }

    fn stdout(&self) -> Cow<'_, str> {
        Cow::Borrowed("")
    }

    fn stderr(&self) -> Cow<'_, str> {
        Cow::Borrowed("")
    }
}

impl OutputLike for Output {
    fn status(&self) -> ExitStatus {
        self.status
    }

    fn stdout(&self) -> Cow<'_, str> {
        String::from_utf8_lossy(&self.stdout)
    }

    fn stderr(&self) -> Cow<'_, str> {
        String::from_utf8_lossy(&self.stderr)
    }
}

impl OutputLike for Utf8Output {
    fn status(&self) -> ExitStatus {
        self.status
    }

    fn stdout(&self) -> Cow<'_, str> {
        Cow::Borrowed(&self.stdout)
    }

    fn stderr(&self) -> Cow<'_, str> {
        Cow::Borrowed(&self.stderr)
    }
}
