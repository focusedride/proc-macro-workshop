#![feature(extend_one)]
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::Visibility;
use syn::{braced, punctuated::Punctuated, token, Field, Token};
use syn::{parse_macro_input, Ident, Type};

#[proc_macro_derive(Builder, attributes(builder))]
pub fn derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    parse_macro_input!(input as Mystruct).build_macro()
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
            let field_name = f.ident.as_ref().unwrap();
            if f.attrs.len() != 0 {
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
            } else {
                let arg_type = if vec!["args", "env"].contains(&field_name.to_string().as_str()) {
                    quote! { Vec<String>}
                } else {
                    quote! { String }
                };
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
