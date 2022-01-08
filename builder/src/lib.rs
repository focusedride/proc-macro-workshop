// use std::collections::HashMap;
#![feature(extend_one)]
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::LitStr;
use syn::Visibility;
use syn::{braced, punctuated::Punctuated, token, Field, Token};
use syn::{parse::Parse, parse_macro_input, Ident, Type};

#[proc_macro_derive(Builder, attributes(builder))]
pub fn derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as Mystruct).build_macro();
    // println!("{:?}", input);
    input
}

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
                        args: None,
                        env: None,
                        current_dir: None,
                    }
                }
            }
        }
    }

    fn struct_command_builder(&self) -> TokenStream {
        quote! {
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

        quote! {
            impl CommandBuilder {
                fn executable(&mut self, executable: String) -> &mut Self {
                    self.executable = Some(executable);
                    self
                }
                fn args(&mut self, args: Vec<String>) -> &mut Self {
                    self.args = Some(args);
                    self
                }
                fn env(&mut self, env: Vec<String>) -> &mut Self {
                    self.env = Some(env);
                    self
                }
                fn current_dir(&mut self, current_dir: String) -> &mut Self {
                    self.current_dir = Some(current_dir);
                    self
                }

                pub fn build(&mut self) -> Result<#name, Box<dyn std::error::Error>> {
                    Ok(#name {
                        #(#build_fields),*
                    })
                }
            }
        }
    }
}

#[allow(dead_code, unused_imports)]
#[derive(Debug)]
struct Mystruct {
    _vis: Visibility,
    struct_token: Token![struct],
    ident: Ident,
    brace_token: token::Brace,
    fields: Punctuated<Field, Token![,]>,
    setters: Vec<proc_macro2::TokenStream>,
    fn_build_fields: Vec<proc_macro2::TokenStream>,
}

#[allow(dead_code, unused_variables)]
#[derive(Debug)]
struct Myattr {
    ident: Ident,
    eq: token::Eq,
    fn_name: LitStr,
}

impl Parse for Myattr {
    #[allow(dead_code, unused_variables)]
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ident: Ident = input.parse()?;
        match ident.to_string().as_str() {
            "each" => Ok(Myattr {
                ident,
                eq: input.parse()?,
                fn_name: input.parse()?,
            }),
            bad => Err(syn::parse::Error::new(
                Span::call_site(),
                format!("wtf, bad attribute: {}", bad),
            )),
        }
    }
}

impl Parse for Mystruct {
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
        // let mut attr_fields = HashMap::new();
        for f in fields.iter() {
            if f.attrs.len() != 0 {
                for a in f.attrs.iter() {
                    let fn_name = a.parse_args::<Myattr>().unwrap().fn_name;
                    let t = proc_macro2::TokenStream::from(quote! {let x = 5;});
                    setters.extend_one(t);
                }
            } else {
                let t = proc_macro2::TokenStream::from(quote! {let x = 5;});
                setters.extend_one(t);
            }
            let build_fields = match &f.ty {
                Type::Path(p) => {
                    let field_name = f.ident.as_ref().unwrap();
                    let strname = &field_name.to_string();
                    match p.path.segments.last() {
                        Some(s) if s.ident == "Option" => quote! {
                            #field_name: self.#field_name.clone()
                        },
                        Some(_) => quote! {
                            #field_name: self.#field_name.clone().ok_or(#strname)?
                        },
                        None => panic!("bad"),
                    }
                }
                _ => unimplemented!("were not there yet"),
            };

            fn_build_fields.extend_one(build_fields)
        }

        Ok(Mystruct {
            _vis,
            struct_token,
            ident,
            brace_token: brace,
            fields,
            fn_build_fields,
            setters,
        })
    }
}
