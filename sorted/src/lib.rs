use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;

#[proc_macro_attribute]
pub fn sorted(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::Item);
    // let args = parse_macro_input!(args as syn::Item);
    if let Err(e) = sorted_variants(input) {
        TokenStream::from(e.to_compile_error())
    } else {
        quote! {}.into()
    }
}

fn sorted_variants(input: syn::Item) -> syn::Result<proc_macro2::TokenStream> {
    if let syn::Item::Enum(_x) = input {
        Ok(quote! {})
    } else {
        Err(syn::Error::new(
            proc_macro2::Span::call_site(),
            "expected enum or match expression",
        ))
    }
}
