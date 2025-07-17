use std::path::PathBuf;
use crate::{AsyncTask, AsyncStream};

pub struct FileLoader<T> {
    pub iterator: Box<dyn Iterator<Item = T>>,
}

pub struct FileLoaderBuilder<T> {
    pattern: Option<String>,
    recursive: bool,
    iterator: Option<Box<dyn Iterator<Item = T>>>,
}

impl FileLoader<PathBuf> {
    // Semantic entry point
    pub fn files_matching(pattern: &str) -> FileLoaderBuilder<PathBuf> {
        let paths: Vec<PathBuf> = glob::glob(pattern)
            .expect("Failed to read glob pattern")
            .filter_map(Result::ok)
            .collect();
            
        FileLoaderBuilder {
            pattern: Some(pattern.to_string()),
            recursive: false,
            iterator: Some(Box::new(paths.into_iter())),
        }
    }
}

impl<T: 'static> FileLoaderBuilder<T> {
    pub fn recursive(mut self, recursive: bool) -> Self {
        self.recursive = recursive;
        self
    }
    
    pub fn filter<F>(mut self, f: F) -> Self
    where
        F: Fn(&T) -> bool + 'static,
    {
        if let Some(iter) = self.iterator {
            self.iterator = Some(Box::new(iter.filter(f)));
        }
        self
    }
    
    pub fn map<U, F>(self, f: F) -> FileLoaderBuilder<U>
    where
        F: Fn(T) -> U + 'static,
        U: 'static,
    {
        FileLoaderBuilder {
            pattern: self.pattern,
            recursive: self.recursive,
            iterator: self.iterator.map(|iter| {
                Box::new(iter.map(f)) as Box<dyn Iterator<Item = U>>
            }),
        }
    }
    
    // Terminal method - loads all files
    pub fn load(self) -> Vec<T> {
        self.iterator
            .expect("Iterator must be provided")
            .collect()
    }
    
    // Terminal method - loads as stream
    pub fn stream(self) -> AsyncStream<T> 
    where
        T: crate::async_task::NotResult + Send + 'static,
    {
        // Would create a stream from the iterator
        AsyncStream::default()
    }
    
    // Terminal method - processes each file
    pub fn process<F, U>(self, processor: F) -> AsyncTask<Vec<U>>
    where
        F: Fn(T) -> U + Send + 'static,
        U: crate::async_task::NotResult + Send + 'static,
        U: Send + 'static,
        T: Send + 'static,
    {
        let items: Vec<T> = self.load();
        AsyncTask::spawn(move || {
            items.into_iter().map(processor).collect()
        })
    }
    
    // Terminal method with handler
    pub fn on_each<F>(self, handler: F) -> AsyncTask<()>
    where
        F: Fn(T) + Send + 'static,
        T: Send + 'static,
    {
        let items: Vec<T> = self.load();
        AsyncTask::spawn(move || {
            for item in items {
                handler(item);
            }
        })
    }
}