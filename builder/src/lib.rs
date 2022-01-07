#[allow(dead_code)]
use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

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

                pub fn build(self) -> Result<#name, Box<dyn std::error::Error>> {
                    let e: Box<dyn std::error::Error> = String::from("test").into();
                        Ok(#name {
                            executable: self.executable.clone().ok_or("test")?,
                            args: self.args.clone().ok_or("tet")?,
                            env: self.env.clone().ok_or("ff")?,
                            current_dir: self.current_dir.clone().ok_or("ff")?
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
