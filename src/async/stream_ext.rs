//! Stream extension traits for async stream processing

use super::{AsyncStream, AsyncTask, NotResult};
use core::future::Future;
use std::vec::Vec;

type Error = Box<dyn std::error::Error + Send + Sync>;

//────────────────────────────────────────────────────────────────────────────
// StreamExt – Fluent ops for AsyncStream<T>
//────────────────────────────────────────────────────────────────────────────

/// Extension trait for streams that provides additional combinators for async stream operations.
pub trait StreamExt<T>: Sized + 'static {
    /// Processes each result in the stream with the provided function.
    fn on_result<F>(self, f: F) -> AsyncStream<T>
    where
        F: FnMut(Result<T, Error>) -> Result<T, Error> + Send + 'static,
        T: NotResult;

    /// Processes each chunk in the stream with the provided function.
    fn on_chunk<F, U>(self, f: F) -> AsyncStream<U>
    where
        F: FnMut(Result<T, Error>) -> U + Send + 'static,
        U: Send + 'static + NotResult;

    /// Processes each error in the stream with the provided function.
    fn on_error<F>(self, f: F) -> AsyncStream<T>
    where
        F: FnMut(Error) + Send + 'static,
        T: NotResult;

    /// Applies a function to each item in the stream without consuming it.
    fn tap_each(self, f: impl FnMut(&T) + Send + 'static) -> AsyncStream<T>
    where
        T: NotResult;

    /// Applies a function to each item while forwarding the original item.
    fn tee_each(self, f: impl FnMut(T) + Send + 'static) -> AsyncStream<T>
    where
        T: NotResult;

    /// Maps each item in the stream to a new type using the provided function.
    fn map_stream<U: Send + 'static + NotResult>(
        self,
        f: impl FnMut(T) -> U + Send + 'static,
    ) -> AsyncStream<U>;

    /// Filters items in the stream based on a predicate function.
    fn filter_stream(self, f: impl FnMut(&T) -> bool + Send + 'static) -> AsyncStream<T>
    where
        T: NotResult;

    /// Partitions the stream into chunks of the specified size.
    fn partition_chunks(self, chunk_size: usize) -> AsyncStream<Vec<T>>
    where
        Vec<T>: NotResult;

    /// Terminates the stream by collecting all values into a Vec.
    fn collect(self) -> AsyncTask<Vec<T>>
    where
        Vec<T>: NotResult;

    /// Terminates the stream, running an async function for each item.
    fn await_result<F, Fut>(self, f: F) -> AsyncTask<()>
    where
        F: FnMut(T) -> Fut + Send + 'static,
        Fut: Future<Output = Result<(), Error>> + Send + 'static;

    /// Terminates the stream, running an async function for each item, discarding result.
    fn await_ok<F, Fut>(self, f: F) -> AsyncTask<()>
    where
        F: FnMut(T) -> Fut + Send + 'static,
        Fut: Future<Output = ()> + Send + 'static;
}

// Implementation of StreamExt for AsyncStream
impl<T: Clone + Send + 'static + NotResult> StreamExt<T> for AsyncStream<T> {
    fn on_result<F>(self, mut f: F) -> AsyncStream<T>
    where
        F: FnMut(Result<T, Error>) -> Result<T, Error> + Send + 'static,
    {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

        tokio::spawn(async move {
            use futures::StreamExt;
            let mut stream = self;
            while let Some(item) = stream.next().await {
                match f(Ok(item)) {
                    Ok(v) => {
                        if tx.send(v).is_err() {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
        });

        AsyncStream::new(rx)
    }

    fn on_chunk<F, U>(self, mut f: F) -> AsyncStream<U>
    where
        Self: Send + 'static,
        F: FnMut(Result<T, Error>) -> U + Send + 'static,
        U: Send + 'static + NotResult,
    {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

        tokio::spawn(async move {
            use futures::StreamExt;
            let mut stream = self;
            while let Some(item) = stream.next().await {
                let result = f(Ok(item));
                if tx.send(result).is_err() {
                    break;
                }
            }
        });

        AsyncStream::new(rx)
    }

    fn on_error<F>(self, _f: F) -> AsyncStream<T>
    where
        F: FnMut(Error) + Send + 'static,
    {
        // Since AsyncStream<T> doesn't carry errors in the stream itself,
        // this is a no-op that just passes through
        self
    }

    fn tap_each(self, mut f: impl FnMut(&T) + Send + 'static) -> AsyncStream<T> {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

        tokio::spawn(async move {
            use futures::StreamExt;
            let mut stream = self;
            while let Some(item) = stream.next().await {
                f(&item);
                if tx.send(item).is_err() {
                    break;
                }
            }
        });

        AsyncStream::new(rx)
    }

    fn tee_each(self, mut f: impl FnMut(T) + Send + 'static) -> AsyncStream<T> {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

        tokio::spawn(async move {
            use futures::StreamExt;
            let mut stream = self;
            while let Some(item) = stream.next().await {
                f(item.clone());
                if tx.send(item).is_err() {
                    break;
                }
            }
        });

        AsyncStream::new(rx)
    }

    fn map_stream<U: Send + 'static + NotResult>(
        self,
        mut f: impl FnMut(T) -> U + Send + 'static,
    ) -> AsyncStream<U> {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

        tokio::spawn(async move {
            use futures::StreamExt;
            let mut stream = self;
            while let Some(item) = stream.next().await {
                if tx.send(f(item)).is_err() {
                    break;
                }
            }
        });

        AsyncStream::new(rx)
    }

    fn filter_stream(self, mut f: impl FnMut(&T) -> bool + Send + 'static) -> AsyncStream<T> {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

        tokio::spawn(async move {
            use futures::StreamExt;
            let mut stream = self;
            while let Some(item) = stream.next().await {
                if f(&item) {
                    if tx.send(item).is_err() {
                        break;
                    }
                }
            }
        });

        AsyncStream::new(rx)
    }

    fn partition_chunks(self, chunk_size: usize) -> AsyncStream<Vec<T>>
    where
        Vec<T>: NotResult,
    {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

        tokio::spawn(async move {
            use futures::StreamExt;
            let mut stream = self;
            let mut buffer = Vec::with_capacity(chunk_size);

            while let Some(item) = stream.next().await {
                buffer.push(item);
                if buffer.len() >= chunk_size {
                    let chunk = std::mem::replace(&mut buffer, Vec::with_capacity(chunk_size));
                    if tx.send(chunk).is_err() {
                        break;
                    }
                }
            }

            // Send remaining items
            if !buffer.is_empty() {
                let _ = tx.send(buffer);
            }
        });

        AsyncStream::new(rx)
    }

    fn collect(self) -> AsyncTask<Vec<T>> {
        self.collect_async()
    }

    fn await_result<F, Fut>(self, mut f: F) -> AsyncTask<()>
    where
        F: FnMut(T) -> Fut + Send + 'static,
        Fut: Future<Output = Result<(), Error>> + Send + 'static,
    {
        AsyncTask::from_future(async move {
            use futures::StreamExt;
            let mut stream = self;
            while let Some(item) = stream.next().await {
                if let Err(_) = f(item).await {
                    break;
                }
            }
        })
    }

    fn await_ok<F, Fut>(self, mut f: F) -> AsyncTask<()>
    where
        F: FnMut(T) -> Fut + Send + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        AsyncTask::from_future(async move {
            use futures::StreamExt;
            let mut stream = self;
            while let Some(item) = stream.next().await {
                f(item).await;
            }
        })
    }
}
