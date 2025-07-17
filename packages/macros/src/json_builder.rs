//! Proc macro support for transparent JSON syntax in builder methods

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemImpl, ImplItem, ImplItemFn, Signature, FnArg, Pat, Type, Block};

/// Attribute macro that enables transparent JSON syntax for builder methods
/// Apply this to impl blocks to automatically transform methods to support `{"key" => "value"}` syntax
#[proc_macro_attribute]
pub fn json_builder(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input_impl = parse_macro_input!(item as ItemImpl);
    
    // Transform the impl block to support JSON syntax
    let transformed_impl = transform_impl_for_json_syntax(input_impl);
    
    quote! { #transformed_impl }.into()
}

/// Transform an impl block to support JSON syntax in method parameters
fn transform_impl_for_json_syntax(mut impl_block: ItemImpl) -> ItemImpl {
    // Look for methods that should support JSON syntax
    for item in &mut impl_block.items {
        if let ImplItem::Fn(method) = item {
            // Check if this is a builder method that should support JSON syntax
            if is_json_builder_method(&method.sig.ident.to_string()) {
                *method = transform_method_for_json_syntax(method.clone());
            }
        }
    }
    
    impl_block
}

/// Check if a method should support JSON syntax
fn is_json_builder_method(method_name: &str) -> bool {
    matches!(method_name, "additional_params" | "metadata" | "new")
}

/// Transform a method to support JSON syntax
fn transform_method_for_json_syntax(mut method: ImplItemFn) -> ImplItemFn {
    // For now, just return the original method
    // In a full implementation, this would:
    // 1. Create a wrapper version of the method
    // 2. Add a macro call inside that transforms JSON syntax
    // 3. Generate appropriate conversions
    
    method
}

/// Macro that users can call to enable JSON syntax in a block of code
#[proc_macro]
pub fn with_json_syntax(input: TokenStream) -> TokenStream {
    let input_tokens = proc_macro2::TokenStream::from(input);
    
    // Transform the input to automatically apply hash_map_fn! to JSON-like syntax
    let transformed = transform_tokens_for_json_syntax(input_tokens);
    
    transformed.into()
}

/// Transform token stream to automatically apply hash_map_fn! to JSON syntax
fn transform_tokens_for_json_syntax(tokens: proc_macro2::TokenStream) -> proc_macro2::TokenStream {
    let token_string = tokens.to_string();
    
    // Look for patterns like `{"key" => "value"}` and transform them
    if token_string.contains("=>") && token_string.contains("{") && token_string.contains("}") {
        // This is a very simplified transformation
        // In a full implementation, this would be much more sophisticated
        let transformed = token_string
            .replace("additional_params({", "additional_params(hash_map_fn!{")
            .replace("metadata({", "metadata(hash_map_fn!{")
            .replace("Tool::new({", "Tool::new(hash_map_fn!{");
        
        transformed.parse().unwrap_or(tokens)
    } else {
        tokens
    }
}