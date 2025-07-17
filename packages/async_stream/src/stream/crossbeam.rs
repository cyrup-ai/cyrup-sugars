//! Crossbeam + async-channel hybrid implementation for compute-heavy workloads

use async_channel;
use futures::Stream;
use std::pin::Pin;
use std::task::{Context, Poll};
use sugars_async_task::NotResult;

/// Generic async stream wrapper for streaming operations with Crossbeam + async-channel
///
/// IMPORTANT: AsyncStream must never contain Result types - all error handling
/// should be done internally before sending items to the stream.
///
/// This implementation uses async-channel for async compatibility with crossbeam
/// for compute-heavy parallel processing.
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
    /// Create a new AsyncStream from a crossbeam receiver
    pub fn new(receiver: channel::Receiver<T>) -> Self {
        Self {
            receiver,
            waker: Arc::new(Mutex::new(None)),
        }
    }

    /// Create an AsyncStream from a futures Stream
    pub fn from_stream<S>(stream: S) -> super::AsyncTask<Vec<T>>
    where
        S: Stream<Item = T> + Send + 'static,
        T: Send + 'static,
    {
        let (tx, rx) = channel::unbounded();

        std::thread::spawn(move || {
            use futures::StreamExt;
            let rt = tokio::runtime::Handle::current();
            rt.block_on(async move {
                let mut stream = std::pin::pin!(stream);
                while let Some(item) = stream.next().await {
                    if tx.send(item).is_err() {
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

            while let Ok(item) = receiver.recv() {
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
        // Store the waker for crossbeam channel notifications
        if let Ok(mut waker_guard) = self.waker.lock() {
            *waker_guard = Some(cx.waker().clone());
        }

        match self.receiver.try_recv() {
            Ok(item) => Poll::Ready(Some(item)),
            Err(channel::TryRecvError::Empty) => Poll::Pending,
            Err(channel::TryRecvError::Disconnected) => Poll::Ready(None),
        }
    }
}
