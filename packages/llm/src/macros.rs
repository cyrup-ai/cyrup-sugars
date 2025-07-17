//! Internal macros for JSON object syntax

/// Wrapper around hash_map_fn! that converts to std HashMap with serde_json::Value
#[macro_export]
macro_rules! json_params {
    ({$($key:expr => $value:expr),* $(,)?}) => {
        || {
            use std::collections::HashMap;
            use serde_json::Value;
            // Use the hashbrown macro to get the syntax, then convert
            let hb_map = $crate::hash_map_fn!{$($key => $value),*}();
            let mut map: HashMap<String, Value> = HashMap::new();
            for (k, v) in hb_map {
                map.insert(k.to_string(), Value::String(v.to_string()));
            }
            map
        }
    };
}
