//! Tests for async features

#[cfg(feature = "async")]
mod async_tokio_tests {
    use cyrup_sugars::{AsyncTask, AsyncResult, NotResult};
    
    #[tokio::test]
    async fn test_async_tokio_async_task_basic() {
        let task = AsyncTask::from_value(42);
        let result = task.await;
        assert_eq!(result, 42);
    }

    #[tokio::test] 
    async fn test_async_tokio_async_task_from_future() {
        let future = async { 42 };
        let task = AsyncTask::from_future(future);
        let result = task.await;
        assert_eq!(result, 42);
    }

    #[tokio::test]
    async fn test_async_tokio_not_result() {
        // Test that NotResult prevents Result types
        fn accepts_not_result<T: NotResult>(_value: T) -> T {
            _value
        }
        
        // This should compile
        let _good = accepts_not_result(42);
        let _good = accepts_not_result("hello");
        let _good = accepts_not_result(vec![1, 2, 3]);
        
        // These would not compile (which is what we want):
        // let _bad = accepts_not_result(Ok::<i32, ()>(42));
        // let _bad = accepts_not_result(Err::<i32, ()>(()));
    }

    #[tokio::test]
    async fn test_async_tokio_async_result() {
        let success: AsyncResult<i32, String> = AsyncResult::ok(42);
        assert!(success.is_ok());
        match success.into_inner() {
            Ok(val) => assert_eq!(val, 42),
            Err(_) => panic!("Should be Ok"),
        }
        
        let error: AsyncResult<i32, String> = AsyncResult::err("failed".to_string());
        assert!(error.is_err());
        match error.into_inner() {
            Ok(_) => panic!("Should be Err"),
            Err(e) => assert_eq!(e, "failed"),
        }
    }

    #[cfg(feature = "tokio-async")]
    #[tokio::test]
    async fn test_async_tokio_stream_ext() {
        use cyrup_sugars::StreamExt;
        use tokio_stream::{self as stream, StreamExt as TokioStreamExt};
        
        let stream = stream::iter(vec![1, 2, 3, 4, 5]);
        let doubled: Vec<i32> = stream.map(|x| x * 2).collect().await;
        assert_eq!(doubled, vec![2, 4, 6, 8, 10]);
    }

    #[cfg(feature = "tokio-async")]
    #[tokio::test]
    async fn test_async_tokio_future_ext() {
        use cyrup_sugars::FutureExt;
        
        let task = AsyncTask::from_value(42);
        let result = task.tap_ok(|val| println!("Got value: {}", val)).await;
        assert_eq!(result, 42);
    }
}

#[cfg(feature = "std-async")]
mod async_std_tests {
    use cyrup_sugars::{AsyncTask, AsyncResult};

    #[tokio::test]
    async fn test_async_std_basic() {
        let task = AsyncTask::from_value(42);
        let result = task.await;
        assert_eq!(result, 42);
    }

    #[tokio::test]
    async fn test_async_std_stream() {
        use cyrup_sugars::StreamExt;
        use futures::stream;
        use futures::StreamExt as FuturesStreamExt;
        
        let stream = stream::iter(vec![1, 2, 3]);
        let collected: Vec<i32> = stream.collect().await;
        assert_eq!(collected, vec![1, 2, 3]);
    }
}

#[cfg(feature = "crossbeam-async")]
mod async_crossbeam_tests {
    use cyrup_sugars::{AsyncTask, AsyncResult};

    #[tokio::test]
    async fn test_async_crossbeam_basic() {
        let task = AsyncTask::from_value(42);
        let result = task.await;
        assert_eq!(result, 42);
    }

    #[tokio::test]
    async fn test_async_crossbeam_stream() {
        use cyrup_sugars::StreamExt;
        use futures::stream;
        use futures::StreamExt as FuturesStreamExt;
        
        let stream = stream::iter(vec![1, 2, 3]);
        let collected: Vec<i32> = stream.collect().await;
        assert_eq!(collected, vec![1, 2, 3]);
    }
}

#[cfg(all(feature = "async", feature = "collections"))]
mod async_collections_integration_tests {
    use cyrup_sugars::{AsyncTask, OneOrMany, ZeroOneOrMany};

    #[tokio::test]
    async fn test_async_collections_integration() {
        let collection = OneOrMany::many(vec![1, 2, 3]).expect("test data");
        let task = AsyncTask::from_value(collection);
        let result = task.await;
        assert_eq!(result.len(), 3);
        assert_eq!(result.first(), &1);
    }

    #[tokio::test]
    async fn test_async_collections_zero_one_many() {
        let collection = ZeroOneOrMany::many(vec![1, 2, 3]);
        let task = AsyncTask::from_value(collection);
        let result = task.await;
        assert_eq!(result.len(), 3);
        assert_eq!(result.first(), Some(&1));
    }
}