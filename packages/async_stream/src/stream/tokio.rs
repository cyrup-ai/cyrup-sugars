//! Tokio-based async stream implementation

use futures::Stream;
use std::pin::Pin;
use std::task::{Context, Poll};
use sugars_async_task::{AsyncTask, NotResult};
use sugars_collections::ZeroOneOrMany;
use tokio::sync::mpsc;

/// Generic async stream wrapper for streaming operations with Tokio
///
/// IMPORTANT: AsyncStream must never contain Result types - all error handling
/// should be done internally before sending items to the stream.
pub struct AsyncStream<T>
where
    T: NotResult, // T cannot be any Result type
{
    receiver: mpsc::UnboundedReceiver<T>,
}

impl<T> AsyncStream<T>
where
    T: NotResult, // T cannot be any Result type
{
    /// Create a new AsyncStream from an unbounded receiver
    pub fn new(receiver: mpsc::UnboundedReceiver<T>) -> Self {
        Self { receiver }
    }

    /// Create an AsyncStream from a futures Stream
    pub fn from_stream<S>(stream: S) -> AsyncTask<Vec<T>>
    where
        S: Stream<Item = T> + Send + 'static,
        T: Send + 'static,
    {
        let (tx, rx) = tokio::sync::oneshot::channel();

        tokio::spawn(async move {
            use futures::StreamExt;
            let mut stream = std::pin::pin!(stream);
            let mut items = Vec::new();
            while let Some(item) = stream.next().await {
                items.push(item);
            }
            let _ = tx.send(items);
        });

        AsyncTask::new(ZeroOneOrMany::one(rx))
    }

    /// Collect all items from the stream into a Vec
    pub fn collect_async(self) -> AsyncTask<Vec<T>>
    where
        T: Send + 'static,
    {
        let (tx, rx) = tokio::sync::oneshot::channel();
        let mut receiver = self.receiver;

        tokio::spawn(async move {
            let mut items = Vec::new();
            while let Some(item) = receiver.recv().await {
                items.push(item);
            }
            let _ = tx.send(items);
        });

        AsyncTask::new(ZeroOneOrMany::one(rx))
    }
}

impl<T> Stream for AsyncStream<T>
where
    T: NotResult,
{
    type Item = T;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.receiver.poll_recv(cx)
    }
}
