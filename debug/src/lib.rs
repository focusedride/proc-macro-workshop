#![allow(unused_variables, dead_code, unused_imports)]
use std::any::Any;

use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::Attribute;
use syn::DeriveInput;

#[proc_macro_derive(CustomDebug, attributes(debug))]
pub fn derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::ItemStruct);
    let i = &input.ident;
    let u = syn::LitStr::new(
        input.ident.to_string().as_str(),
        proc_macro2::Span::call_site(),
    );
    let b = input.fields.build_debug_struct_fields();
    if let Some(g) = input.generics.type_params().last() {
        let ident = &g.ident;
        if let Some(q) = input.fields.get_phantom_field() {
            proc_macro::TokenStream::from(quote! {
                impl<#ident> std::fmt::Debug for #i<#ident>
                where
                    PhantomData<#ident>: Debug
                {
                    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        f.debug_struct(#u)
                        #b
                        .finish()
                    }
                }
            })
        } else {
            proc_macro::TokenStream::from(quote! {
                impl<#ident: std::fmt::Debug> std::fmt::Debug for #i<#ident> {
                    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        f.debug_struct(#u)
                        #b
                        .finish()
                    }
                }
            })
        }
    } else {
        proc_macro::TokenStream::from(quote! {
            impl std::fmt::Debug for #i {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    f.debug_struct(#u)
                    #b
                    .finish()
                }
            }
        })
    }
}

// type StructGeneric = syn::punctuated::Punctuated<syn::GenericParam, syn::token::Comma>;
// trait X {
//     fn x(self) -> u8;
// }
// impl X for &syn::Generics {
//     fn x(self) -> bool {
//         2
//     }
// }

trait FieldsParser {
    // type PhantomGeneric;
    fn get_phantom_field(&self) -> std::option::Option<syn::Ident>;
    fn build_debug_struct_fields(&self) -> proc_macro2::TokenStream;
    fn debug_field_format(f: &syn::Field) -> syn::Result<proc_macro2::TokenStream>;
}
impl FieldsParser for syn::Fields {
    // type PhantomGeneric = syn::AngleBracketedGenericArguments;
    fn get_phantom_field(&self) -> Option<syn::Ident> {
        for f in self.iter() {
            if let syn::Type::Path(syn::TypePath { qself, path }) = &f.ty {
                if let Some(s) = path.segments.first() {
                    if s.ident.to_string().as_str() == "PhantomData" {
                        return f.ident.clone();
                    }
                }
            }
        }
        None
    }
    fn debug_field_format(f: &syn::Field) -> syn::Result<proc_macro2::TokenStream> {
        let field_name = &f.ident;
        if let Some(a) = f.attrs.first() {
            if let Ok(syn::Meta::NameValue(syn::MetaNameValue { lit, path, .. })) = a.parse_meta() {
                if path.is_ident("debug") {
                    if let syn::Lit::Str(l) = lit {
                        return Ok(quote! { &format_args!(#l, &self.#field_name)});
                    }
                } else {
                    return Err(syn::Error::new_spanned(f, "only debug is implpemented"));
                }
            }
        }
        Ok(quote! { &self.#field_name })
    }
    fn build_debug_struct_fields(&self) -> proc_macro2::TokenStream {
        let fields = self
            .iter()
            .map(|f| {
                let a = syn::LitStr::new(
                    f.ident.as_ref().unwrap().to_string().as_str(),
                    proc_macro2::Span::mixed_site(),
                );
                let c = Self::debug_field_format(&f).unwrap();
                return quote! { .field(#a, #c) };
            })
            .collect::<Vec<_>>();
        quote! {#(#fields)*}.into()
    }
}
