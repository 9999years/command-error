use std::fmt::Debug;
use std::fmt::Display;

use crate::CommandDisplay;

pub struct OutputConversionError {
    pub(crate) command: Box<dyn CommandDisplay>,
    pub(crate) inner: Box<dyn Display>,
}

impl Debug for OutputConversionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OutputConversionError")
            .field("program", &self.command.program())
            .field("inner", &self.inner.to_string())
            .finish()
    }
}

impl Display for OutputConversionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Failed to convert `{}` output: {}",
            self.command.program(),
            self.inner
        )
    }
}
