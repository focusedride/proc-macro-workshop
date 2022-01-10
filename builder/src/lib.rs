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
                executable: std::option::Option<String>,
                args: std::option::Option<Vec<String>>,
                env: std::option::Option<Vec<String>>,
                current_dir: std::option::Option<String>,
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

                pub fn build(&mut self) -> std::result::Result<#name, std::boxed::Box<dyn std::error::Error>> {
                    Ok(#name {
                        #(#build_fields),*
                    })
                }
            }
        }
    }

    fn setter_function_each_variant(fn_name: &syn::LitStr, field_name: &syn::Ident) -> TokenStream {
        let fn_name = Ident::new(fn_name.value().as_str(), Span::call_site());
        quote! {
            fn #fn_name(&mut self, argument: String) -> &mut Self {
                if let Some(ref mut v) = self.#field_name {
                    v.push(argument);
                }
                self
            }
        }
    }

    fn setter_function_all_variant(field_name: &syn::Ident) -> TokenStream {
        let arg_type = match vec!["args", "env"].contains(&field_name.to_string().as_str()) {
            true => quote! { Vec<String>},
            false => quote! { String },
        };
        quote! {
            fn #field_name(&mut self, ar: #arg_type) -> &mut Self {
                self.#field_name = Some(ar);
                self
            }
        }
    }

    fn builder_function_field(
        p: &syn::TypePath,
        field_name: &syn::Ident,
    ) -> syn::Result<TokenStream> {
        match p.path.segments.last() {
            Some(s) if s.ident == "Option" => Ok(quote! {
                #field_name: self.#field_name.clone()
            }),
            Some(_) => {
                let strname = &field_name.to_string();
                Ok(quote! {
                    #field_name: self.#field_name.clone().ok_or(format!("field {} exploded",#strname))?
                })
            }
            None => std::result::Result::Err(::syn::Error::new_spanned(p, "build field error")),
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

type MyAttr = syn::punctuated::Punctuated<syn::MetaNameValue, Token![,]>;
trait EachField {
    fn check_ident(arg: &MyAttr) -> bool;
    fn extract_value(arg: &MyAttr) -> syn::Result<&syn::LitStr>;
}

impl EachField for MyAttr {
    fn check_ident(arg: &MyAttr) -> bool {
        match arg.last() {
            Some(v) => v.path.is_ident("each"),
            None => false,
        }
    }
    fn extract_value(arg: &Self) -> syn::Result<&syn::LitStr> {
        if let syn::Lit::Str(fn_name) = &arg.last().unwrap().lit {
            Ok(fn_name)
        } else {
            let err_msg = r#"expected `builder(each = "...")`"#;
            let span = arg;
            Err(syn::Error::new_spanned(span, err_msg))
        }
    }
}

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
            if let Some(attr) = f.attrs.last() {
                if let Ok(arg) = attr.parse_args_with(MyAttr::parse_separated_nonempty) {
                    if <MyAttr as EachField>::check_ident(&arg) {
                        if let syn::Lit::Str(fn_name) = &arg.last().unwrap().lit {
                            setters.extend_one(Mystruct::setter_function_each_variant(
                                fn_name, field_name,
                            ));
                        }
                    } else {
                        let err_msg = r#"expected `builder(each = "...")`"#;
                        let span = &attr.parse_meta().unwrap();
                        return Err(syn::Error::new_spanned(span, err_msg));
                    }
                }
            } else {
                setters.extend_one(Mystruct::setter_function_all_variant(&field_name))
            }
            match &f.ty {
                Type::Path(p) => {
                    fn_build_fields.extend_one(Mystruct::builder_function_field(p, field_name)?)
                }
                _ => unimplemented!("were not there yet"),
            };
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
