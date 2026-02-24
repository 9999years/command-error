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
use futures_util::future::BoxFuture;
use tokio::process::Child;
use utf8_command::Utf8Output;

/// NB: No `try_wait` methods, and I should also revisit those definitions in `command_error`.
pub trait AsyncChildExt: Sized {
    /// The error type returned from methods on this trait.
    type Error: From<Error>;

    #[track_caller]
    fn output_checked_as<O, R, E, F>(
        self,
        succeeded: F,
    ) -> impl Future<Output = Result<R, E>> + Send + 'static
    where
        O: Debug + OutputLike + TryFrom<Output> + Send + Sync + 'static,
        <O as TryFrom<Output>>::Error: Display + Send + Sync,
        E: From<Self::Error>,
        F: FnOnce(OutputContext<O>) -> BoxFuture<'static, Result<R, E>> + Send + 'static;

    #[track_caller]
    fn output_checked_with<O, E, F>(
        self,
        succeeded: F,
    ) -> impl Future<Output = Result<O, Self::Error>> + Send + 'static
    where
        O: Debug + OutputLike + TryFrom<Output> + Send + Sync + 'static,
        <O as TryFrom<Output>>::Error: Display + Send + Sync,
        E: Debug + Display,
        E: Send + Sync + 'static,
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
    fn output_checked(self) -> impl Future<Output = Result<Output, Self::Error>> + Send + 'static {
        self.output_checked_with(|output: &Output| {
            Box::pin(async move {
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
        self,
    ) -> impl Future<Output = Result<Utf8Output, Self::Error>> + Send + 'static {
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
        self,
        succeeded: F,
    ) -> impl Future<Output = Result<Utf8Output, Self::Error>> + Send + 'static
    where
        E: Display + Debug + Send + Sync + 'static,
        F: FnOnce(&Utf8Output) -> BoxFuture<'_, Result<(), Option<E>>> + Send + 'static,
    {
        self.output_checked_with(succeeded)
    }

    #[track_caller]
    fn wait_checked_as<R, E, F, F2>(
        &mut self,
        succeeded: F,
    ) -> impl Future<Output = Result<R, E>> + Send
    where
        E: From<Self::Error>,
        F: FnOnce(OutputContext<ExitStatus>) -> F2 + Send + 'static,
        F2: Future<Output = Result<R, E>> + Send + 'static;

    #[track_caller]
    fn wait_checked_with<E, F, F2>(
        &mut self,
        succeeded: F,
    ) -> impl Future<Output = Result<ExitStatus, Self::Error>> + Send
    where
        E: Debug + Display + Send + Sync + 'static,
        F: FnOnce(ExitStatus) -> F2 + Send + 'static,
        F2: Future<Output = Result<(), Option<E>>> + Send + 'static,
    {
        self.wait_checked_as(
            async move |context| match succeeded(context.status()).await {
                Ok(()) => Ok(context.status()),
                Err(user_error) => Err(Error::from(match user_error {
                    Some(error) => context.output_error().with_message(Box::new(error)),
                    None => context.output_error(),
                })
                .into()),
            },
        )
    }

    #[track_caller]
    fn wait_checked(&mut self) -> impl Future<Output = Result<ExitStatus, Self::Error>> + Send {
        self.wait_checked_with(async |status| {
            if status.success() {
                Ok(())
            } else {
                Err(None::<String>)
            }
        })
    }

    fn log(&self);
}

impl AsyncChildExt for ChildContext<Child> {
    type Error = Error;

    #[track_caller]
    #[allow(clippy::manual_async_fn)] // Don't even joke...
    fn output_checked_as<O, R, E, F>(
        self,
        succeeded: F,
    ) -> impl Future<Output = Result<R, E>> + Send + 'static
    where
        O: Debug + OutputLike + Send + Sync + TryFrom<Output> + 'static,
        <O as TryFrom<Output>>::Error: Display + Send + Sync,
        E: From<Self::Error>,
        F: FnOnce(OutputContext<O>) -> BoxFuture<'static, Result<R, E>> + Send + 'static,
    {
        async move {
            self.log();
            let (child, command) = self.into_child_and_command();
            match child.wait_with_output().await {
                Ok(output) => match output.try_into() {
                    Ok(output) => succeeded(OutputContext::new(output, command)).await,
                    Err(error) => Err(Error::from(OutputConversionError::new(
                        command,
                        Box::new(error),
                    ))
                    .into()),
                },
                Err(inner) => Err(Error::from(ExecError::new(command, inner)).into()),
            }
        }
    }

    #[track_caller]
    #[allow(clippy::manual_async_fn)] // Don't even joke...
    fn wait_checked_as<R, E, F, F2>(
        &mut self,
        succeeded: F,
    ) -> impl Future<Output = Result<R, E>> + Send
    where
        E: From<Self::Error>,
        F: FnOnce(OutputContext<ExitStatus>) -> F2 + Send + 'static,
        F2: Future<Output = Result<R, E>> + Send + 'static,
    {
        async move {
            self.log();
            let command = self.command_boxed().clone();
            match self.child_mut().wait().await {
                Ok(status) => succeeded(OutputContext::new(status, command)).await,
                Err(inner) => Err(Error::from(ExecError::new(command, inner)).into()),
            }
        }
    }

    fn log(&self) {
        tracing::debug!(command = %self.command(), "Executing command");
    }
}
