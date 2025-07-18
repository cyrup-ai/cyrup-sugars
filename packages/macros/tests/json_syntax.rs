use sugars_macros::json_syntax;
use std::collections::HashMap;

// Mock hash_map_fn for testing purposes
#[macro_export]
macro_rules! hash_map_fn {
    ($($key:expr => $value:expr),* $(,)? ) => {{
        let mut map = ::std::collections::HashMap::new();
        $( map.insert($key.to_string(), $value.to_string()); )*
        map
    }};
}

#[test]
fn test_with_turbofish_syntax() {
    #[json_syntax]
    fn my_function() -> HashMap<String, String> {
        let _ = Vec::<i32>::new();
        HashMap::new()
    }
    assert_eq!(my_function(), HashMap::new());
}

#[test]
fn test_with_json_syntax() {
    #[json_syntax]
    fn my_function() -> HashMap<String, String> {
        let map = {
            "key1" => "value1",
            "key2" => "value2"
        };
        map
    }

    let mut expected = HashMap::new();
    expected.insert("key1".to_string(), "value1".to_string());
    expected.insert("key2".to_string(), "value2".to_string());

    assert_eq!(my_function(), expected);
}

#[test]
fn test_with_combined_syntax() {
    #[json_syntax]
    fn my_function() -> HashMap<String, String> {
        let _ = Vec::<i32>::new();
        let map = {
            "key1" => "value1",
            "key2" => "value2"
        };
        map
    }

    let mut expected = HashMap::new();
    expected.insert("key1".to_string(), "value1".to_string());
    expected.insert("key2".to_string(), "value2".to_string());

    assert_eq!(my_function(), expected);
}
