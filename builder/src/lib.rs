use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse_macro_input, Data, DeriveInput, Type};

struct MacroBuilder {
    struct_command: DeriveInput,
}

impl MacroBuilder {
    fn from(input: DeriveInput) -> Self {
        MacroBuilder {
            struct_command: input,
        }
    }

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
        let name = &self.struct_command.ident;
        let build_fields = match &self.struct_command.data {
            Data::Struct(d) => d
                .fields
                .iter()
                .map(|f| match &f.ty {
                    Type::Path(p) => {
                        let field_name = f.ident.as_ref().unwrap();
                        let outer_type = p.path.segments.last().unwrap();
                        if &outer_type.ident == "Option" {
                            // outer_type
                            //     .arguments
                            //     .to_token_stream()
                            //     .into_iter()
                            //     .filter(|t| t.to_string().len() > 1)
                            //     .collect();
                            quote! {
                                #field_name: self.#field_name.clone()
                            }
                        } else {
                            outer_type.to_token_stream();
                            quote! {
                                #field_name: self.#field_name.clone().ok_or("test")?
                            }
                        }
                    }
                    _ => unimplemented!("were not there yet"),
                })
                .collect::<Vec<_>>(),
            _ => unimplemented!("were not there yet"),
        };
        // for f in &build_fields {
        //     println!("{}", f);
        //     // println!("{:?}", f);
        // }

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
                    let e: Box<dyn std::error::Error> = String::from("test").into();

                    Ok(#name {
                        #(#build_fields),*
                    })
                }
            }

        }
    }
}

#[proc_macro_derive(Builder)]
pub fn derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    MacroBuilder::from(input).build_macro()
}
