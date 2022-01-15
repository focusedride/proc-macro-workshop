#![allow(unused_imports, unused_variables, dead_code, unused_parens)]
use std::convert::TryInto;

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
    let data_struct = input.build_struct_with_data_array();
    let accessors = input.build_accessors().unwrap();
    quote! {#data_struct #types #accessors}.into()
}

trait Bitfield {
    fn build_bit_types(&self) -> TokenStream;
    fn build_struct_with_data_array(&self) -> TokenStream;
    fn build_accessors(&self) -> syn::Result<TokenStream>;
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
        let struct_name = &self.ident;
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
            #[derive(Debug)]
            pub struct #struct_name {
                data: [u8; #size],
            }

            impl #struct_name {
                fn new() -> Self {
                    Self { data: [0; #size] }
                }
            }
        }
    }
    fn build_accessors(&self) -> syn::Result<TokenStream> {
        let struct_name = &self.ident;
        // let current_bit = 1;
        let mut section = (0, 0);
        let accessors = self
            .fields
            .iter()
            .map(move |f| {
                let span = proc_macro2::Span::call_site();
                let i = f.ident.as_ref().unwrap().clone();
                let t = f.ty.clone();
                if let syn::Type::Path(syn::TypePath { path, .. }) = &f.ty {
                    let size = path
                        .segments
                        .first()
                        .unwrap()
                        .ident
                        .to_string()
                        .trim_start_matches("B")
                        .parse::<u16>()
                        .unwrap();
                    if section.0 == 0 {
                        section.1 = size;
                    } else {
                        section.0 = 1 + section.1;
                        section.1 += 1 + size;
                    }
                    let set_i = syn::Ident::new(format!("set_{}", i.to_string()).as_str(), span);
                    let get_i = syn::Ident::new(format!("get_{}", i.to_string()).as_str(), span);
                    let range = syn::ExprRange {
                        attrs: vec![],
                        from: Some(Box::new(syn::Expr::Lit(syn::ExprLit {
                            lit: syn::Lit::Int(syn::LitInt::new(
                                (section.0 / 8).to_string().as_str(),
                                span,
                            )),
                            attrs: vec![],
                        }))),
                        limits: syn::RangeLimits::Closed(syn::token::DotDotEq {
                            ..Default::default()
                        }),
                        to: Some(Box::new(syn::Expr::Lit(syn::ExprLit {
                            lit: syn::Lit::Int(syn::LitInt::new(
                                (section.1 / 8).to_string().as_str(),
                                span,
                            )),
                            attrs: vec![],
                        }))),
                    };
                    let l = (section.1 / 8) - (section.0 / 8) + 1;
                    let bl = (section.1) - (section.0);
                    // distance from right edge
                    let drre = 8- section.1.rem_euclid(8);
                    // distance from left edge
                    // let dfle = section.1 - section.0 +  drre + bl - 1;
                    let bl = syn::Lit::Int(syn::LitInt::new(
                        bl.to_string().as_str(),
                        span,
                    ));

                    // let dfle = syn::Lit::Int(syn::LitInt::new(
                    //     format!("{}u8", dfle).as_str(),
                    //     span,
                    // ));
                    let drre = syn::Lit::Int(syn::LitInt::new(
                        format!("{}u8", drre).as_str(),
                        span,
                    ));

                    let mapped_type: syn::Type = match l {
                        1 => syn::parse_str("u8").unwrap(),
                        2 => syn::parse_str("u16").unwrap(),
                        3 | 4 => syn::parse_str("u32").unwrap(),
                        5..=8 => syn::parse_str("u64").unwrap(),
                        _ => syn::parse_str("usize").unwrap(),
                    };
                    let l = syn::Lit::Int(syn::LitInt::new(
                        ((section.1 / 8) - (section.0 / 8) + 1).to_string().as_str(),
                        span,
                    ));
                    let lr = (section.0 % 8, section.1 % 8);
                    // BUG invert https://stackoverflow.com/questions/26004263/set-the-i-th-bit-to-zero
                    let getter = quote! {
                        fn #get_i(&self) -> u64 {
                            let chunk = unsafe {
                                std::mem::transmute_copy::<[u8; #l], #mapped_type>(self.data[#range].try_into().unwrap())
                            };
                            let needle = #mapped_type::MAX >> #bl;
                            return ((chunk >> #drre) & (needle)) as u64;
                            // >> u64::from_str_radix(&format!("{:#010b}", (0b11111111 >> 5 << 3) & 0b010101001).trim_start_matches("0b")[2..=4], 2).unwrap()
                        }
                    };
                    let setter = quote! {
                        fn #set_i(&mut self, v: #mapped_type) {
                            eprintln!("value = {}", v );
                            eprintln!("value = \t\t{:#010b}", v );
                            let chunk = unsafe {
                                std::mem::transmute_copy::<[u8; #l], #mapped_type>(self.data[#range].try_into().unwrap())
                            };
                            let drill = #mapped_type::MAX >> #drre << #bl;
                            let drilled_chunk = chunk & !drill;
                            let v = v << #drre;
                            let new_chunk = v | drilled_chunk;
                            let new_chunk = unsafe {
                                std::mem::transmute_copy::<#mapped_type, [u8; #l]>(new_chunk.try_into().unwrap())
                            };
                            eprintln!("mvalue = \t\t{:#010b}", v );
                            eprintln!("chunk = \t\t{:#010b}", chunk);
                            eprintln!("drill = \t\t{:#010b}", drill);
                            eprintln!("dchunk = \t\t{:#010b}", drilled_chunk);
                            eprintln!("nchunk = \t\t{:#010b}", v | drilled_chunk);

                            for i in #range {
                                self.data[i] = new_chunk[i];
                            }
                        }
                    };
                    return quote! {#getter #setter};
                }
                quote! {}
            })
            .collect::<Vec<TokenStream>>();

        Ok(quote! {
            impl #struct_name {

                #(#accessors)*
            }
        })
    }
}
