use proc_macro::TokenStream;
use syn::parse_macro_input;

mod wiring;

#[proc_macro_derive(Wiring, attributes(wiring))]
pub fn derive_wiring(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::DeriveInput);
    wiring::derive_wiring_impl(input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
