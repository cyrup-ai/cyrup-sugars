use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{
    parse_macro_input, visit_mut::{self, VisitMut}, Expr, ExprCall, ExprPath, ItemFn, PathArguments,
};

struct JsonSyntaxTransformer;

impl VisitMut for JsonSyntaxTransformer {
    fn visit_expr_mut(&mut self, expr: &mut Expr) {
        if let Expr::Call(call_expr) = expr {
            // Fix turbofish syntax: `foo<T>()` -> `foo::<T>()`
            if let Expr::Path(ExprPath { qself: None, path, .. }) = &*call_expr.func {
                if let Some(segment) = path.segments.last() {
                    if let PathArguments::AngleBracketed(_) = &segment.arguments {
                        let mut new_path = path.clone();
                        if let Some(last_mut) = new_path.segments.last_mut() {
                            let args = std::mem::replace(&mut last_mut.arguments, PathArguments::None);
                            let new_func_tokens = quote! { #new_path::#args };
                            if let Ok(new_func) = syn::parse2(new_func_tokens) {
                                call_expr.func = Box::new(new_func);
                            }
                        }
                    }
                }
            }

            // Transform JSON-like blocks: `method({ "k" => "v" })` -> `method(sugars_collections::hash_map!{ "k" => "v" })`
            if let Some(first_arg) = call_expr.args.first_mut() {
                if let Expr::Block(block_expr) = first_arg {
                    if quote!(#block_expr).to_string().contains(" => ") {
                        let new_arg_tokens = quote! { sugars_collections::hash_map! #block_expr };
                        if let Ok(new_arg) = syn::parse2(new_arg_tokens) {
                            *first_arg = new_arg;
                        }
                    }
                }
            }
        }

        // Standard pre-order traversal.
        visit_mut::visit_expr_mut(self, expr);
    }
}

#[proc_macro_attribute]
pub fn json_syntax(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut function = parse_macro_input!(item as ItemFn);
    let mut transformer = JsonSyntaxTransformer;
    transformer.visit_block_mut(&mut function.block);
    function.into_token_stream().into()
}
