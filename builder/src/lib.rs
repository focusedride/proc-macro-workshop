use proc_macro::TokenStream;
use syn::{parse_macro_input, Data, DeriveInput, Fields, __private::ToTokens};
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let a = input.attrs;
    for x in a {
        println!("{}", x.to_token_stream());
    }
    match input.data {
        Data::Struct(ref s) => match s.fields {
            Fields::Named(ref fields) => {
                for f in fields.named.iter() {
                    println!("{:?}{:?}", f.ident, f.to_token_stream());
    let expanded = quote! {
        pub struct CommandBuilder {
            executable: Option<String>,
            args: Option<Vec<String>>,
            env: Option<Vec<String>>,
            current_dir: Option<String>,
        }


        impl Command {
            pub fn builder() -> CommandBuilder {
                CommandBuilder {
                    executable: None,
                    args: None,
                    env: None,
                    current_dir: None,
                }
            }
            _ => println!("else"),
        },
        _ => println!("else"),
    }
    println!(
        "{}{}{}",
        input.vis.to_token_stream(),
        input.generics.to_token_stream(),
        input.ident.to_token_stream()
    );
        }

    };

    TokenStream::from(expanded)
}
