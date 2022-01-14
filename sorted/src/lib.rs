use proc_macro::TokenStream;
use quote::quote;

#[proc_macro_attribute]
pub fn sorted(args: TokenStream, input: TokenStream) -> TokenStream {
    let _ = args;
    let _ = input;

    quote! {}.into()
}
