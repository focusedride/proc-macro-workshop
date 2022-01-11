#![allow(unused_variables, dead_code, unused_imports)]
use std::any::Any;

use proc_macro::{Span, TokenStream};
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
    if let Some(g) = input.generics.type_params().last() {
        let ident = &g.ident;
        if let Some(q) = input.fields.contains_phantom_field() {
            proc_macro::TokenStream::from(quote! {
                impl<#ident> std::fmt::Debug for #i<#ident>
                where
                    PhantomData<#ident>: Debug
                {
                    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        f.debug_struct(#u)
                        .field("string", &self.string)
                        .field("bitmask", &format_args!("0b{:08b}", &self.bitmask))
                        .finish()
                    }
                }
            })
        } else {
            proc_macro::TokenStream::from(quote! {
                impl<#ident: std::fmt::Debug> std::fmt::Debug for #i<#ident> {
                    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        f.debug_struct(#u)
                        .field("value", &self.value)
                        .field("bitmask", &format_args!("0b{:08b}", &self.bitmask))
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
                    .field("name", &self.name)
                    .field("bitmask", &format_args!("0b{:08b}", &self.bitmask))
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
    type PhantomGeneric;
    fn contains_phantom_field(&self) -> std::option::Option<syn::Ident>;
}
impl FieldsParser for syn::Fields {
    type PhantomGeneric = syn::AngleBracketedGenericArguments;
    fn contains_phantom_field(&self) -> Option<syn::Ident> {
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
}
