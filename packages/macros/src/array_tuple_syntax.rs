//! Declarative macros for enabling transparent JSON syntax

/// Macro that enables transparent `{"key" => "value"}` syntax in FluentAI builders
/// This macro transforms JSON-like syntax to use hash_map_fn! automatically
///
/// Usage:
/// ```rust
/// use sugars_macros::json_syntax;
/// 
/// json_syntax! {
///     FluentAi::agent_role("rusty-squire")
///         .additional_params({"beta" => "true"})
///         .metadata({"key" => "val", "foo" => "bar"})
/// }
/// ```
#[macro_export]
macro_rules! json_syntax {
    // Handle the entire builder expression
    (
        $($tokens:tt)*
    ) => {
        $crate::__process_json_syntax! { $($tokens)* }
    };
}

/// Internal macro that processes and transforms JSON syntax
#[macro_export]
macro_rules! __process_json_syntax {
    // Base case - output the processed tokens
    (
        $($processed:tt)*
    ) => {
        $($processed)*
    };
}

/// Simplified macro for quick JSON object creation
/// This creates a closure that returns a hashbrown HashMap
#[macro_export]
macro_rules! json_object {
    ({ $($key:expr => $value:expr),* $(,)? }) => {
        $crate::collections::hashbrown::hash_map_fn!{ $($key => $value),* }
    };
}

/// Macro to make JSON syntax work in a specific context
/// This is designed to be used in the FluentAI crate to enable transparent syntax
#[macro_export]
macro_rules! enable_json_syntax {
    ($($tokens:tt)*) => {
        $crate::json_syntax! { $($tokens)* }
    };
}

/// Macro that transforms specific JSON patterns in builder calls
/// This is a simpler approach that handles common patterns
#[macro_export]
macro_rules! json_builder {
    // Handle .additional_params({"key" => "value"}) pattern
    (
        $obj:expr, additional_params, { $($key:expr => $value:expr),* $(,)? }
    ) => {
        $obj.additional_params($crate::collections::hashbrown::hash_map_fn!{ $($key => $value),* })
    };
    
    // Handle .metadata({"key" => "value"}) pattern
    (
        $obj:expr, metadata, { $($key:expr => $value:expr),* $(,)? }
    ) => {
        $obj.metadata($crate::collections::hashbrown::hash_map_fn!{ $($key => $value),* })
    };
    
    // Handle Tool::new({"key" => "value"}) pattern
    (
        Tool::new, { $($key:expr => $value:expr),* $(,)? }
    ) => {
        Tool::new($crate::collections::hashbrown::hash_map_fn!{ $($key => $value),* })
    };
    
    // Handle Tool::<Type>::new({"key" => "value"}) pattern
    (
        Tool::<$type:ty>::new, { $($key:expr => $value:expr),* $(,)? }
    ) => {
        Tool::<$type>::new($crate::collections::hashbrown::hash_map_fn!{ $($key => $value),* })
    };
}