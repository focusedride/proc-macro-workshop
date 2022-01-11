#![allow(unused_variables, dead_code, unused_imports)]
use proc_macro::{Span, TokenStream};
use quote::quote;
use syn::Attribute;

#[proc_macro_derive(CustomDebug, attributes(debug))]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);
    let i = &input.ident;
    let u = syn::LitStr::new(
        input.ident.to_string().as_str(),
        proc_macro2::Span::call_site(),
    );
    let a = quote! {
        impl std::fmt::Debug for #i {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.debug_struct(#u)
                .field("name", &self.name)
                .field("bitmask", &format_args!("0b{:08b}", &self.bitmask))
                .finish()
            }
        }
    }
    .into();
    a
}
