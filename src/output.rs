use std::borrow::Borrow;
use std::fmt::Debug;
use std::fmt::Display;
use std::process::ExitStatus;

use crate::CommandDisplay;
use crate::Error;
use crate::OutputError;
use crate::OutputLike;

pub struct Output<O> {
    pub(crate) output: O,
    pub(crate) command: Box<dyn CommandDisplay>,
}

impl<O> Output<O>
where
    O: OutputLike + 'static,
{
    pub fn into_output(self) -> O {
        self.output
    }

    pub fn output(&self) -> &O {
        &self.output
    }

    pub fn status(&self) -> ExitStatus {
        self.output.status()
    }

    pub fn command(&self) -> &dyn CommandDisplay {
        self.command.borrow()
    }

    pub fn error(self) -> Error {
        Error::from(OutputError::new(self.command, Box::new(self.output)))
    }

    pub fn error_msg<E>(self, message: E) -> Error
    where
        E: Debug + Display + 'static,
    {
        Error::from(
            OutputError::new(self.command, Box::new(self.output))
                .with_user_error(Some(Box::new(message))),
        )
    }

    pub(crate) fn maybe_error_msg<E>(self, message: Option<E>) -> Error
    where
        E: Debug + Display + 'static,
    {
        let ret = OutputError::new(self.command, Box::new(self.output));
        Error::from(match message {
            Some(message) => ret.with_user_error(Some(Box::new(message))),
            None => ret,
        })
    }
}
