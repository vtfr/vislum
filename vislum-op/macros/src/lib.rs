use proc_macro::TokenStream;
use syn::parse_macro_input;

mod node;
mod value;

#[proc_macro_derive(Value, attributes(value))]
pub fn derive_value(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::DeriveInput);

    value::derive_value_impl(input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[proc_macro_derive(Node, attributes(node, input, output))]
pub fn derive_node(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::DeriveInput);

    node::derive_reflect_impl(input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
