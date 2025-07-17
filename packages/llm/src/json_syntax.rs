//! Provides transparent JSON object syntax support

/// Macro that enables {"key" => "value"} syntax for builder methods
/// This wraps method calls and automatically applies hash_map_fn! where needed
#[macro_export]
macro_rules! json_params {
    // Match the additional_params pattern
    ($builder:expr, additional_params, {$($k:expr => $v:expr),* $(,)?}) => {
        $builder.additional_params($crate::hash_map_fn!{$($k => $v),*})
    };
    
    // Match the metadata pattern  
    ($builder:expr, metadata, {$($k:expr => $v:expr),* $(,)?}) => {
        $builder.metadata($crate::hash_map_fn!{$($k => $v),*})
    };
}