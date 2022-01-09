// use std::collections::HashMap;
#![feature(extend_one)]
use std::vec;

#[allow(unused_imports)]
// use proc_macro::bridge::server::SourceFile;
#[allow(unused_imports)]
use proc_macro2::{Span, TokenStream};
#[allow(unused_imports)]
use quote::quote;
#[allow(unused_imports)]
use quote::ToTokens;
#[allow(unused_imports)]
use syn::bracketed;
#[allow(unused_imports)]
use syn::parenthesized;
#[allow(unused_imports)]
use syn::parse::Parser;
#[allow(unused_imports)]
use syn::Attribute;
#[allow(unused_imports)]
use syn::LitStr;
#[allow(unused_imports)]
use syn::Meta;
#[allow(unused_imports)]
use syn::Visibility;
#[allow(unused_imports)]
use syn::{braced, punctuated::Punctuated, token, Field, Token};
#[allow(unused_imports)]
use syn::{parse::Parse, parse_macro_input, Ident, Type};

#[proc_macro_derive(Builder, attributes(builder))]
pub fn derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as Mystruct);
    // let mcro = input.build_macro();
    expand(input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
    // println!("{}", input);
}

// mod my_trait {
//     use proc_macro2::TokenStream;
//     use syn::{DeriveInput, Result};

fn expand(input: Mystruct) -> Result<proc_macro2::TokenStream, syn::Error> {
    Ok(input.build_macro().into())
}
// }

impl Mystruct {
    fn build_macro(self) -> proc_macro::TokenStream {
        let a = self.impl_command();
        let b = self.struct_command_builder();
        let c = self.impl_command_builder();
        proc_macro::TokenStream::from(quote! { #a #b #c })
    }

    fn impl_command(&self) -> TokenStream {
        quote! {
            impl Command {
                pub fn builder() -> CommandBuilder {
                    CommandBuilder {
                        executable: None,
                        args: Some(vec![]),
                        env: Some(vec![]),
                        current_dir: None,
                    }
                }
            }
        }
    }

    fn struct_command_builder(&self) -> TokenStream {
        quote! {
            #[derive(Debug)]
            pub struct CommandBuilder {
                executable: Option<String>,
                args: Option<Vec<String>>,
                env: Option<Vec<String>>,
                current_dir: Option<String>,
            }
        }
    }

    fn impl_command_builder(&self) -> TokenStream {
        let name = &self.ident;
        let build_fields = &self.fn_build_fields;
        let setters = &self.setters;

        quote! {
            impl CommandBuilder {
                #(#setters)*

                pub fn build(&mut self) -> Result<#name, Box<dyn std::error::Error>> {
                    Ok(#name {
                        #(#build_fields),*
                    })
                }
            }
        }
    }
}

#[derive(Debug)]
struct Mystruct {
    _vis: Visibility,
    _struct_token: Token![struct],
    ident: Ident,
    _brace_token: token::Brace,
    _fields: Punctuated<Field, Token![,]>,
    setters: Vec<proc_macro2::TokenStream>,
    fn_build_fields: Vec<proc_macro2::TokenStream>,
}

#[derive(Debug)]
struct Myattr {
    // _pound: token::Pound,
    // _brackets: token::Bracket,
    // _name: Ident,
    // _parens: token::Paren,
    _ident: syn::Ident,
    _eq: syn::token::Eq,
    #[allow(dead_code)]
    fn_name: syn::LitStr,
}

impl syn::parse::Parse for Myattr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        // let _content;
        // let _inner_content;
        // let _pound = input.parse()?;
        // let _brackets = bracketed!(_content in input);
        // let _name: Ident = input.parse()?;
        // let _parens = parenthesized!(_inner_content in input);
        // input.call()
        let ident: syn::Ident = input.parse()?;
        match ident.to_string().as_str() {
            "each" => Ok(Myattr {
                // _pound,
                // _brackets,
                // _name,
                // _parens,
                _ident: ident,
                _eq: input.parse()?,
                fn_name: input.parse()?,
            }),

            _bad => std::result::Result::Err(syn::Error::new_spanned(
                ident,
                "expected `builder(each = \"...\")`",
            )),
        }
    }
}

#[allow(dead_code, unused_variables)]
fn parse_attrs(tokens: TokenStream) -> syn::Result<String> {
    Ok("".to_string())
}

fn is_ident_eq_to_each(arg: &MyAttr) -> bool {
    match arg.last() {
        Some(v) => v.path.is_ident("each"),
        None => false,
    }
}
type MyAttr = syn::punctuated::Punctuated<syn::MetaNameValue, Token![,]>;

impl syn::parse::Parse for Mystruct {
    #[allow(dead_code, unused_variables)]
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let _vis = input.parse()?;
        let struct_token = input.parse()?;
        let ident = input.parse()?;
        let content;
        let brace = braced!(content in input);
        let fields = content.parse_terminated(Field::parse_named)?;
        let mut setters = vec![];
        let mut fn_build_fields = vec![];
        for f in fields.iter() {
            //     attrs: &Vec<syn::Attribute>,
            // ) -> Result<String, proc_macro2::TokenStream> {
            //     if let std::option::Option::Some(attr) = attrs.last() {
            //         if let std::result::Result::Ok(syn::Meta::List(list)) = attr.parse_meta() {
            //             if list.path.is_ident("builder") {
            let field_name = f.ident.as_ref().unwrap();
            if f.attrs.len() != 0 {
                // if let Ok(syn::Meta::List(li)) = f.attrs.last().unwrap().parse_() {
                //     dbg!(li.nested);
                // }

                if let Some(attr) = f.attrs.last() {
                    if let Ok(arg) = attr.parse_args_with(
                    Punctuated::<syn::MetaNameValue, syn::token::Comma>::parse_separated_nonempty,
                ) {
                    if is_ident_eq_to_each(&arg) {
                        if let syn::Lit::Str(fn_name) = &arg.last().unwrap().lit {
                            let fn_name = Ident::new(fn_name.value().as_str(), Span::call_site());
                            let t = quote! {
                                fn #fn_name(&mut self, argument: String) -> &mut Self {
                                    // let a = argument.clone();
                                    if let Some(ref mut v) = self.#field_name {
                                        v.push(argument);
                                    }
                                    self
                                }
                            };
                            setters.extend_one(t);
                        }
                    } else {
                        return Err(syn::Error::new_spanned(&attr.parse_meta().unwrap(), r#"expected `builder(each = "...")`"#));
                    }
                }
                }
                // let d = args.first().unwrap();
                // let (path, lit) = (
                //     d.path.segments.first().unwrap().ident.to_string(),
                //     d.lit.into_token_stream().to_string(),
                // );
                //
                // let lol = Punctuated::<syn::Ident, syn::LitStr>::parse_separated_nonempty(d); //::<syn::Ident, syn::token::Eq>::parse_terminated(d);
                // dbg!(d);
                // dbg!(lit.into_token_stream().to_string());
                // dbg!(&path.segments.first().unwrap().ident.to_string());

                // f.attrs.last().unwrap().parse_args_with(Punctuated::<>);
                // for a in f.attrs.iter() {
                //     // dbg!(Attribute::parse_meta(&a).unwrap());
                //     // match Attribute::parse_meta(&a) {
                //     //     Ok(syn::Meta::NameValue(v)) => dbg!(v),
                //     //     Err(e) => return Err(e),
                //     //     _ => return Err(syn::Error::new(Span::call_site(), "ff")),
                //     // };
                //     match a.parse_args::<Myattr>() {
                //         Ok(s) => {
                //             let fn_name = Ident::new(s.fn_name.value().as_str(), Span::call_site());
                //             // println!("{} {}", fn_name, field_name);
                //             let t = quote! {
                //                 fn #fn_name(&mut self, argument: String) -> &mut Self {
                //                     // let a = argument.clone();
                //                     if let Some(ref mut v) = self.#field_name {
                //                         v.push(argument);
                //                     }
                //                     self
                //                 }
                //             };
                //             setters.extend_one(t);
                //         }
                //         Err(e) => return Err(e),
                //     }
                //     // let attr = a.parse_args::<Myattr>().unwrap();
                //     // let fn_name = Ident::new(attr.fn_name.value().as_str(), Span::call_site());
                //     // // println!("{} {}", fn_name, field_name);
                //     // let t = quote! {
                //     //     fn #fn_name(&mut self, argument: String) -> &mut Self {
                //     //         // let a = argument.clone();
                //     //         if let Some(ref mut v) = self.#field_name {
                //     //             v.push(argument);
                //     //         }
                //     //         self
                //     //     }
                //     // };
                //     // setters.extend_one(t);
                // }
            } else {
                let arg_type = if vec!["args", "env"].contains(&field_name.to_string().as_str()) {
                    quote! { Vec<String>}
                } else {
                    quote! { String }
                };
                // println!("{} {}", arg_type, field_name);
                let t = quote! {
                    fn #field_name(&mut self, ar: #arg_type) -> &mut Self {
                        self.#field_name = Some(ar);
                        self
                    }
                };
                setters.extend_one(t);
            }
            let build_fields = match &f.ty {
                Type::Path(p) => {
                    let strname = &field_name.to_string();
                    // println!("{} {}", strname, field_name);
                    match p.path.segments.last() {
                        Some(s) if s.ident == "Option" => quote! {
                            #field_name: self.#field_name.clone()
                        },
                        Some(_) => quote! {
                            #field_name: self.#field_name.clone().ok_or(format!("fuck-{}",#strname))?
                        },
                        None => panic!("bad"),
                    }
                }
                _ => unimplemented!("were not there yet"),
            };

            // fn_build_fields.extend_one(initialize_field(f));
            fn_build_fields.extend_one(build_fields)
        }

        Ok(Mystruct {
            _vis,
            _struct_token: struct_token,
            ident,
            _brace_token: brace,
            _fields: fields,
            fn_build_fields,
            setters,
        })
    }
}
// fn field_attr_builder_each(f: &syn::Field) -> Option<Result<syn::Ident, syn::Error>> {
//     if f.attrs.is_empty() {
//         return None;
//     }

//     if let syn::Meta::List(meta_list) = f.attrs[0].parse_meta().ok()? {
//         let err = syn::Error::new_spanned(meta_list.clone(), r#"expected `builder(each = "...")`"#);

//         if format!("{}", meta_list.ident) == "builder" {
//             // found builder, but nothing after that
//             if meta_list.nested.is_empty() {
//                 return Some(Err(err));
//             }

//             if let syn::NestedMeta::Meta(syn::Meta::NameValue(syn::MetaNameValue {
//                 ident,
//                 lit,
//                 ..
//             })) = &meta_list.nested[0]
//             {
//                 if format!("{}", ident) == "each" {
//                     if let syn::Lit::Str(lit_str) = lit {
//                         return Some(Ok(Ident::new(&lit_str.value(), ident.span())));
//                     } else {
//                         return Some(Err(err));
//                     }
//                 } else {
//                     return Some(Err(err));
//                 }
//             } else {
//                 return Some(Err(err));
//             }
//         } else {
//             return Some(Err(err));
//         }
//     }

//     None
// }

#[allow(dead_code, unused_variables)]
fn extract_each_attr_value(
    attrs: &Vec<syn::Attribute>,
) -> Result<String, proc_macro2::TokenStream> {
    if let std::option::Option::Some(attr) = attrs.last() {
        if let std::result::Result::Ok(syn::Meta::List(list)) = attr.parse_meta() {
            if list.path.is_ident("builder") {
                if let std::option::Option::Some(syn::NestedMeta::Meta(syn::Meta::NameValue(
                    syn::MetaNameValue {
                        path,
                        lit: syn::Lit::Str(lit_str),
                        ..
                    },
                ))) = list.nested.last()
                {
                    if path.is_ident("each") {
                        return std::result::Result::Ok(lit_str.value());
                    } else {
                        println!("not each");
                        return std::result::Result::Err(
                            syn::Error::new_spanned(list, "expected `builder(each = \"...\")`")
                                .to_compile_error(),
                        );
                    }
                }
            }
        }
    }
    unreachable!("you should've check for attrs emptyness before")
}
