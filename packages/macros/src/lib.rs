//! Proc macros for transparent JSON syntax in builder patterns

use proc_macro::TokenStream;
use quote::quote;

/// Attribute macro that automatically transforms JSON syntax in function bodies
///
/// This macro finds patterns like `.method([("key", "value")])` and transforms them
/// to work with array tuple syntax automatically.
///
/// Usage:
/// ```ignore
/// #[array_tuple_syntax]  
/// fn main() {
///     let builder = FluentAi::agent_role("example")
///         .additional_params([("beta", "true")])  // <- array tuple syntax
///         .metadata([("key", "val")]);            // <- array tuple syntax
/// }
/// ```
#[proc_macro_attribute]
pub fn array_tuple_syntax(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // For now, just pass through the input unchanged
    // The transformation happens in the builder methods themselves
    item
}

/// Creates a closure that returns a hashbrown HashMap from key-value pairs
///
/// This replaces the macro_rules! version to work in proc-macro crates
///
/// Usage:
/// ```rust
/// use sugars_macros::hash_map_fn;
///
/// let map_fn = hash_map_fn!{"key" => "value", "foo" => "bar"};
/// let map = map_fn();
/// ```
#[proc_macro]
pub fn hash_map_fn(input: TokenStream) -> TokenStream {
    // Convert the input to a string and manually parse key => value pairs
    let input_str = input.to_string();

    // Transform "key" => "value" pairs to ("key", "value") tuples
    let parts: Vec<&str> = input_str.split(',').collect();
    let mut tuple_pairs = Vec::new();

    for part in parts {
        let trimmed = part.trim();
        if let Some(arrow_pos) = trimmed.find(" => ") {
            let key = trimmed[..arrow_pos].trim();
            let value = trimmed[arrow_pos + 4..].trim();
            tuple_pairs.push(format!("({}, {})", key, value));
        } else if let Some(arrow_pos) = trimmed.find("=>") {
            let key = trimmed[..arrow_pos].trim();
            let value = trimmed[arrow_pos + 2..].trim();
            tuple_pairs.push(format!("({}, {})", key, value));
        }
    }

    let tuple_str = tuple_pairs.join(", ");
    let parsed_tokens: proc_macro2::TokenStream = tuple_str.parse().unwrap_or_default();

    quote! {
        || {
            <::hashbrown::HashMap::<_, _> as ::core::iter::FromIterator<_>>::from_iter([
                #parsed_tokens
            ])
        }
    }
    .into()
}
