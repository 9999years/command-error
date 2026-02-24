// This will eventually be upstreamed to `command_error`.
#![allow(dead_code)]

use std::fmt::Debug;
use std::fmt::Display;
use std::process::ExitStatus;
use std::process::Output;

use command_error::ChildContext;
use command_error::Error;
use command_error::ExecError;
use command_error::OutputContext;
use command_error::OutputConversionError;
use command_error::OutputLike;
use command_error::Utf8ProgramAndArgs;
use futures_util::future::BoxFuture;
use tokio::process::Command;
use utf8_command::Utf8Output;

/// Like [`command_error::CommandExt`], but async.
pub trait AsyncCommandExt: Sized {
    type Error: From<Error> + Send + Sync;
    type Child;

    #[track_caller]
    fn output_checked_as<O, R, E, F>(
        &mut self,
        succeeded: F,
    ) -> impl Future<Output = Result<R, E>> + Send + '_
    where
        O: Debug + OutputLike + TryFrom<Output> + Send + Sync + 'static,
        <O as TryFrom<Output>>::Error: Display + Send + Sync,
        E: From<Self::Error> + Send + Sync,
        F: FnOnce(OutputContext<O>) -> BoxFuture<'static, Result<R, E>> + Send + 'static;

    #[track_caller]
    fn output_checked_with<O, E, F>(
        &mut self,
        succeeded: F,
    ) -> impl Future<Output = Result<O, Self::Error>> + Send + '_
    where
        O: Debug + OutputLike + TryFrom<Output> + Send + Sync + 'static,
        <O as TryFrom<Output>>::Error: Display + Send + Sync,
        E: Debug + Display + Send + Sync + 'static,
        F: FnOnce(&O) -> BoxFuture<'_, Result<(), Option<E>>> + Send + 'static,
    {
        self.output_checked_as(|context| {
            Box::pin(async {
                match succeeded(context.output()).await {
                    Ok(()) => Ok(context.into_output()),
                    Err(user_error) => Err(Error::from(match user_error {
                        Some(error) => context.output_error().with_message(Box::new(error)),
                        None => context.output_error(),
                    })
                    .into()),
                }
            })
        })
    }

    #[track_caller]
    fn output_checked(&mut self) -> impl Future<Output = Result<Output, Self::Error>> + Send + '_ {
        self.output_checked_with(|output: &Output| {
            Box::pin(async {
                if output.status.success() {
                    Ok(())
                } else {
                    Err(None::<String>)
                }
            })
        })
    }

    #[track_caller]
    fn output_checked_utf8(
        &mut self,
    ) -> impl Future<Output = Result<Utf8Output, Self::Error>> + Send + '_ {
        self.output_checked_with_utf8(|output| {
            Box::pin(async {
                if output.status.success() {
                    Ok(())
                } else {
                    Err(None::<String>)
                }
            })
        })
    }

    #[track_caller]
    fn output_checked_with_utf8<E, F>(
        &mut self,
        succeeded: F,
    ) -> impl Future<Output = Result<Utf8Output, Self::Error>> + Send + '_
    where
        E: Display + Debug + Send + Sync + 'static,
        F: FnOnce(&Utf8Output) -> BoxFuture<'_, Result<(), Option<E>>> + Send + 'static,
    {
        self.output_checked_with(succeeded)
    }

    #[track_caller]
    fn status_checked_as<R, E, F>(
        &mut self,
        succeeded: F,
    ) -> impl Future<Output = Result<R, E>> + Send + '_
    where
        E: From<Self::Error>,
        F: FnOnce(OutputContext<ExitStatus>) -> BoxFuture<'static, Result<R, E>> + Send + 'static;

    #[track_caller]
    fn status_checked_with<E, F>(
        &mut self,
        succeeded: F,
    ) -> impl Future<Output = Result<ExitStatus, Self::Error>> + Send + '_
    where
        E: Debug + Display + Send + Sync + 'static,
        F: FnOnce(ExitStatus) -> BoxFuture<'static, Result<(), Option<E>>> + Send + 'static,
    {
        self.status_checked_as(|status| {
            Box::pin(async move {
                match succeeded(status.status()).await {
                    Ok(()) => Ok(status.status()),
                    Err(user_error) => Err(Error::from(match user_error {
                        Some(error) => status.output_error().with_message(Box::new(error)),
                        None => status.output_error(),
                    })
                    .into()),
                }
            })
        })
    }

    #[track_caller]
    fn status_checked(
        &mut self,
    ) -> impl Future<Output = Result<ExitStatus, Self::Error>> + Send + '_ {
        self.status_checked_with(|status| {
            Box::pin(async move {
                if status.success() {
                    Ok(())
                } else {
                    Err(None::<String>)
                }
            })
        })
    }

    #[track_caller]
    fn spawn_checked(&mut self) -> Result<Self::Child, Self::Error>;

    fn log(&self);
}

impl AsyncCommandExt for Command {
    type Child = ChildContext<tokio::process::Child>;

    type Error = Error;

    fn spawn_checked(&mut self) -> Result<Self::Child, Self::Error> {
        let displayed: Utf8ProgramAndArgs = self.as_std().into();
        match self.spawn() {
            Ok(child) => Ok(ChildContext::new(child, Box::new(displayed))),
            Err(inner) => Err(Error::from(ExecError::new(Box::new(displayed), inner))),
        }
    }

    fn log(&self) {
        let command = Utf8ProgramAndArgs::from(self.as_std());
        tracing::debug!(%command, "Executing command");
    }

    fn output_checked_as<O, R, E, F>(
        &mut self,
        succeeded: F,
    ) -> impl Future<Output = Result<R, E>> + Send + '_
    where
        O: Debug + OutputLike + TryFrom<Output> + Send + Sync + 'static,
        <O as TryFrom<Output>>::Error: Display + Send + Sync,
        E: From<Self::Error> + Send + Sync,
        F: FnOnce(OutputContext<O>) -> BoxFuture<'static, Result<R, E>> + Send + 'static,
    {
        self.log();
        let displayed: Utf8ProgramAndArgs = self.as_std().into();
        let displayed = Box::new(displayed);
        async {
            match self.output().await {
                Ok(output) => match output.try_into() {
                    Ok(output) => succeeded(OutputContext::new(output, displayed)).await,
                    Err(error) => Err(Error::from(OutputConversionError::new(
                        displayed,
                        Box::new(error),
                    ))
                    .into()),
                },
                Err(inner) => Err(Error::from(ExecError::new(displayed, inner)).into()),
            }
        }
    }

    fn status_checked_as<R, E, F>(
        &mut self,
        succeeded: F,
    ) -> impl Future<Output = Result<R, E>> + Send + '_
    where
        E: From<Self::Error>,
        F: FnOnce(OutputContext<ExitStatus>) -> BoxFuture<'static, Result<R, E>> + Send + 'static,
    {
        self.log();
        let displayed: Utf8ProgramAndArgs = self.as_std().into();
        let displayed = Box::new(displayed);
        async {
            match self.status().await {
                Ok(status) => succeeded(OutputContext::new(status, displayed)).await,
                Err(inner) => Err(Error::from(ExecError::new(displayed, inner)).into()),
            }
        }
    }
}
