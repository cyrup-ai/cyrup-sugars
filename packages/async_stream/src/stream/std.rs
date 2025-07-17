//! Standard library async stream implementation using async-channel

use async_channel;
use futures::Stream;
use std::pin::Pin;
use std::task::{Context, Poll};
use sugars_async_task::NotResult;

/// Generic async stream wrapper for streaming operations with async-channel
///
/// IMPORTANT: AsyncStream must never contain Result types - all error handling
/// should be done internally before sending items to the stream.
pub struct AsyncStream<T>
where
    T: NotResult, // T cannot be any Result type
{
    receiver: async_channel::Receiver<T>,
}

impl<T> AsyncStream<T>
where
    T: NotResult, // T cannot be any Result type
{
    /// Create a new AsyncStream from an async-channel receiver
    pub fn new(receiver: async_channel::Receiver<T>) -> Self {
        Self { receiver }
    }

    /// Create an AsyncStream from a futures Stream
    pub fn from_stream<S>(stream: S) -> super::AsyncTask<Vec<T>>
    where
        S: Stream<Item = T> + Send + 'static,
        T: Send + 'static,
    {
        let (tx, rx) = async_channel::unbounded();

        std::thread::spawn(move || {
            use futures::StreamExt;
            futures::executor::block_on(async move {
                let mut stream = std::pin::pin!(stream);
                while let Some(item) = stream.next().await {
                    if tx.send(item).await.is_err() {
                        break;
                    }
                }
            });
        });

        AsyncStream::new(rx).collect_async()
    }

    /// Collect all items from the stream into a Vec
    pub fn collect_async(self) -> super::AsyncTask<Vec<T>>
    where
        T: Send + 'static,
    {
        super::AsyncTask::new(async move {
            let mut items = Vec::new();
            let receiver = self.receiver;

            while let Ok(item) = receiver.recv().await {
                items.push(item);
            }

            items
        })
    }
}

impl<T> Stream for AsyncStream<T>
where
    T: NotResult,
{
    type Item = T;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        use std::pin::Pin;

        match Pin::new(&mut self.get_mut().receiver).poll_next(cx) {
            Poll::Ready(Some(item)) => Poll::Ready(Some(item)),
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}
