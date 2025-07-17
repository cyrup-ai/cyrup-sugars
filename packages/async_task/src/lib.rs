#![feature(auto_traits, negative_impls)]

pub mod future_ext;
pub mod task;

pub use future_ext::*;
pub use task::{AsyncTask, NotResult};
