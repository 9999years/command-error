use std::fmt::Debug;
use std::fmt::Display;
use std::process::Output;

use process_wrap::std::StdChildWrapper;
use process_wrap::std::StdCommandWrap;

use crate::ChildContext;
use crate::CommandExt;
use crate::Error;
use crate::ExecError;
use crate::OutputContext;
use crate::OutputConversionError;
use crate::OutputLike;
use crate::Utf8ProgramAndArgs;

impl CommandExt for StdCommandWrap {
    type Error = Error;
    type Child = ChildContext<Box<dyn StdChildWrapper>>;

    fn log(&self) -> Result<(), Self::Error> {
        #[cfg(feature = "tracing")]
        {
            let command: Utf8ProgramAndArgs = self.command().into();
            tracing::debug!(%command, "Executing command");
        }
        Ok(())
    }

    fn output_checked_as<O, R, E>(
        &mut self,
        succeeded: impl Fn(OutputContext<O>) -> Result<R, E>,
    ) -> Result<R, E>
    where
        O: Debug + OutputLike + TryFrom<Output> + Send + Sync + 'static,
        <O as TryFrom<Output>>::Error: Display + Send + Sync,
        E: From<Self::Error> + Send + Sync,
    {
        self.log()?;
        let displayed: Utf8ProgramAndArgs = self.command().into();
        let child = match self.spawn() {
            Ok(child) => child,
            Err(inner) => {
                return Err(Error::from(ExecError::new(Box::new(displayed), inner)).into());
            }
        };

        match child.wait_with_output() {
            Ok(output) => match output.try_into() {
                Ok(output) => succeeded(OutputContext {
                    output,
                    command: Box::new(displayed),
                }),
                Err(error) => Err(Error::from(OutputConversionError {
                    command: Box::new(displayed),
                    inner: Box::new(error),
                })
                .into()),
            },
            Err(inner) => Err(Error::from(ExecError {
                command: Box::new(displayed),
                inner,
            })
            .into()),
        }
    }

    fn status_checked_as<R, E>(
        &mut self,
        succeeded: impl Fn(OutputContext<std::process::ExitStatus>) -> Result<R, E>,
    ) -> Result<R, E>
    where
        E: From<Self::Error>,
    {
        self.log()?;
        let displayed: Utf8ProgramAndArgs = self.command().into();
        let mut child = match self.spawn() {
            Ok(child) => child,
            Err(inner) => {
                return Err(Error::from(ExecError::new(Box::new(displayed), inner)).into());
            }
        };

        match child.wait() {
            Ok(status) => succeeded(OutputContext {
                output: status,
                command: Box::new(displayed),
            }),
            Err(inner) => Err(Error::from(ExecError {
                command: Box::new(displayed),
                inner,
            })
            .into()),
        }
    }

    fn spawn_checked(&mut self) -> Result<Self::Child, Self::Error> {
        let displayed: Utf8ProgramAndArgs = self.command().into();
        match self.spawn() {
            Ok(child) => Ok(ChildContext {
                child,
                command: Box::new(displayed),
            }),
            Err(inner) => Err(Error::from(ExecError {
                command: Box::new(displayed),
                inner,
            })),
        }
    }
}
