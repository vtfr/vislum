use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};
use quote::quote;

#[proc_macro_derive(System)]
pub fn system_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let struct_name = input.ident;

    TokenStream::from(quote! {
        impl vislum_system::System for #struct_name 
        where
            #struct_name: std::any::Any + 'static,
        {
            fn as_any(&self) -> &dyn std::any::Any {
                self
            }

            fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
                self
            }

            fn type_name(&self) -> &str {
                std::any::type_name::<#struct_name>()
            }
        }
    })
}