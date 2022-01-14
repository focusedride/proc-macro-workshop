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
    if let syn::Item::Enum(x) = input {
        if let Some(is_sorted) = x
            .variants
            .iter()
            .collect::<Vec<&syn::Variant>>()
            .windows(2)
            .map(|v| {
                if v[0].ident.to_string() <= v[1].ident.to_string() {
                    None
                } else {
                    Some(v[1])
                }
            })
            .filter(|v| v.is_some())
            .map(|v| v.unwrap())
            .inspect(|x| println!("{:?}", x.ident.to_string()))
            .collect::<Vec<&syn::Variant>>()
            .first()
        {
            let before = x
                .variants
                .iter()
                .find(|x| x.ident.to_string() >= is_sorted.ident.to_string())
                .unwrap();
            return Err(syn::Error::new(
                is_sorted.ident.span(),
                format!(
                    "{} should sort before {}",
                    is_sorted.ident.to_string(),
                    before.ident.to_string()
                ),
            ));
        }

        Ok(quote! {})
    } else {
        Err(syn::Error::new(
            proc_macro2::Span::call_site(),
            "expected enum or match expression",
        ))
    }
}
