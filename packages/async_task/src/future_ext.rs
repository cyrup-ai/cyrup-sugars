//! Future extension traits for async future processing

use crate::task::{AsyncTask, NotResult};
use tokio::sync::oneshot;

//────────────────────────────────────────────────────────────────────────────
// FutureExt – Fluent ops for AsyncTask<T>
//────────────────────────────────────────────────────────────────────────────

/// Extension trait for futures that provides additional combinators for async operations.
pub trait FutureExt<T>: Sized {
    /// Maps the success value of the future to a new type using the provided function.
    fn map<U>(self, f: impl FnOnce(T) -> U + Send + 'static) -> AsyncTask<U>
    where
        U: Send + 'static + NotResult;

    /// Executes a function when the future completes successfully.
    fn on_ok<U>(self, f: impl FnOnce(T) -> U + Send + 'static) -> AsyncTask<U>
    where
        U: Send + 'static + NotResult;

    /// Executes a function when the future encounters an error.
    fn on_error<U>(
        self,
        f: impl FnOnce(oneshot::error::RecvError) -> U + Send + 'static,
    ) -> AsyncTask<U>
    where
        U: Send + 'static + NotResult,
        T: Into<U>;

    /// Handles both success and error cases with a result handler.
    fn on_result<U>(
        self,
        f: impl FnOnce(Result<T, oneshot::error::RecvError>) -> U + Send + 'static,
    ) -> AsyncTask<U>
    where
        U: Send + 'static + NotResult;

    /// Maps the success value while preserving error state.
    fn map_ok<U>(self, f: impl FnOnce(T) -> U + Send + 'static) -> AsyncTask<U>
    where
        U: Send + 'static + NotResult;

    /// Applies a function to the success value without consuming it.
    fn tap_ok(self, f: impl FnOnce(&T) + Send + 'static) -> AsyncTask<T>
    where
        T: NotResult;

    /// Applies a function to the error value without consuming it.
    fn tap_err(self, f: impl FnOnce(&oneshot::error::RecvError) + Send + 'static) -> AsyncTask<T>
    where
        T: NotResult;
}

// Implementation for AsyncTask
impl<T: Send + 'static + NotResult> FutureExt<T> for AsyncTask<T> {
    fn map<U>(self, f: impl FnOnce(T) -> U + Send + 'static) -> AsyncTask<U>
    where
        U: Send + 'static + NotResult,
    {
        self.on_ok(f)
    }

    fn on_ok<U>(self, f: impl FnOnce(T) -> U + Send + 'static) -> AsyncTask<U>
    where
        U: Send + 'static + NotResult,
    {
        AsyncTask::from_future(async move {
            let value = self.await;
            f(value)
        })
    }

    fn on_error<U>(
        self,
        f: impl FnOnce(oneshot::error::RecvError) -> U + Send + 'static,
    ) -> AsyncTask<U>
    where
        U: Send + 'static + NotResult,
        T: Into<U>,
    {
        self.on_result(move |result| match result {
            Ok(value) => value.into(),
            Err(e) => f(e),
        })
    }

    fn on_result<U>(
        self,
        f: impl FnOnce(Result<T, oneshot::error::RecvError>) -> U + Send + 'static,
    ) -> AsyncTask<U>
    where
        U: Send + 'static + NotResult,
    {
        AsyncTask::from_future(async move {
            let result = self.receiver.await;
            f(result)
        })
    }

    fn map_ok<U>(self, f: impl FnOnce(T) -> U + Send + 'static) -> AsyncTask<U>
    where
        U: Send + 'static + NotResult,
    {
        self.on_ok(f)
    }

    fn tap_ok(self, f: impl FnOnce(&T) + Send + 'static) -> AsyncTask<T> {
        AsyncTask::from_future(async move {
            let value = self.await;
            f(&value);
            value
        })
    }

    fn tap_err(self, f: impl FnOnce(&oneshot::error::RecvError) + Send + 'static) -> AsyncTask<T> {
        AsyncTask::from_future(async move {
            match self.receiver.await {
                Ok(value) => value,
                Err(e) => {
                    f(&e);
                    panic!("AsyncTask channel error: {e:?}");
                }
            }
        })
    }
}
