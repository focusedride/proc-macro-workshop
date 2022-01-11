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

            }
        }
    }
    .into();
    a
}
