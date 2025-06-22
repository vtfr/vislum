use proc_macro::TokenStream;
use syn::parse_macro_input;

mod reflect;
mod value;

#[proc_macro_derive(Value, attributes(value))]
pub fn derive_value(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::DeriveInput);

    value::derive_value_impl(input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[proc_macro_derive(Reflect, attributes(reflect, input, output))]
pub fn reflect(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::DeriveInput);

    reflect::derive_reflect_impl(input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
