//! Future extension traits for async future processing

use super::task::{AsyncTask, NotResult};
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
    fn on_err<E>(
        self,
        f: impl FnOnce(oneshot::error::RecvError) -> E + Send + 'static,
    ) -> AsyncTask<T>
    where
        E: Into<oneshot::error::RecvError> + Send + 'static,
        T: NotResult;

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
        let (tx, rx) = oneshot::channel();
        tokio::spawn(async move {
            match self.await {
                Ok(value) => {
                    let _ = tx.send(f(value));
                }
                Err(_) => {
                    // Channel was closed, nothing to do
                }
            }
        });
        AsyncTask::new(rx)
    }

    fn on_err<E>(
        self,
        f: impl FnOnce(oneshot::error::RecvError) -> E + Send + 'static,
    ) -> AsyncTask<T>
    where
        E: Into<oneshot::error::RecvError> + Send + 'static,
    {
        let (tx, rx) = oneshot::channel();
        tokio::spawn(async move {
            match self.await {
                Ok(value) => {
                    let _ = tx.send(value);
                }
                Err(e) => {
                    f(e);
                    // Can't send on error, channel is closed
                }
            }
        });
        AsyncTask::new(rx)
    }

    fn map_ok<U>(self, f: impl FnOnce(T) -> U + Send + 'static) -> AsyncTask<U>
    where
        U: Send + 'static + NotResult,
    {
        self.on_ok(f)
    }

    fn tap_ok(self, f: impl FnOnce(&T) + Send + 'static) -> AsyncTask<T> {
        let (tx, rx) = oneshot::channel();
        tokio::spawn(async move {
            match self.await {
                Ok(value) => {
                    f(&value);
                    let _ = tx.send(value);
                }
                Err(_) => {
                    // Channel was closed, nothing to do
                }
            }
        });
        AsyncTask::new(rx)
    }

    fn tap_err(self, f: impl FnOnce(&oneshot::error::RecvError) + Send + 'static) -> AsyncTask<T> {
        let (tx, rx) = oneshot::channel();
        tokio::spawn(async move {
            match self.await {
                Ok(value) => {
                    let _ = tx.send(value);
                }
                Err(e) => {
                    f(&e);
                    // Can't send on error, channel is closed
                }
            }
        });
        AsyncTask::new(rx)
    }
}
