#![allow(unused_imports, unused_variables, dead_code, unused_parens)]
use proc_macro2::TokenStream;
use quote::quote;
use syn::parse_macro_input;

#[proc_macro_attribute]
pub fn bitfield(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let _ = args;
    let input = parse_macro_input!(input as syn::ItemStruct);
    let types = input.build_bit_types();
    let data = input.build_struct_with_data_array();
    quote! {#data #types}.into()
}

trait Bitfield {
    fn build_bit_types(&self) -> TokenStream;
    fn build_struct_with_data_array(&self) -> TokenStream;
}
impl Bitfield for syn::ItemStruct {
    fn build_bit_types(&self) -> TokenStream {
        let enums: Vec<TokenStream> = (1..65)
            .map(|i| {
                let span = proc_macro2::Span::call_site();
                let type_ident = syn::Ident::new(format!("B{}", i).as_str(), span);
                let value = syn::LitInt::new(&i.to_string(), span);
                quote! {
                    pub enum #type_ident {}
                    impl bitfield::Specifier for #type_ident {
                        const BITS: u8 = #value;
                    }
                }
            })
            .collect();
        quote! { #(#enums)* }
    }
    fn build_struct_with_data_array(&self) -> TokenStream {
        let mut size = 0;
        for f in &self.fields {
            if let syn::Type::Path(syn::TypePath { path, .. }) = &f.ty {
                size += path
                    .segments
                    .first()
                    .unwrap()
                    .ident
                    .to_string()
                    .trim_start_matches("B")
                    .parse::<u16>()
                    .unwrap();
            }
        }
        let span = proc_macro2::Span::call_site();
        let size = syn::LitInt::new(&(size / 8).to_string(), span);
        quote! {
            #[repr(C)]
            pub struct MyFourBytes {
                data: [u8; #size],
            }
        }
    }
}
