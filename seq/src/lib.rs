#![allow(unused_variables, dead_code, unused_imports, unused_parens)]

use std::ops::Deref;
use std::ops::Range;
use std::str::FromStr;

use proc_macro::TokenStream;
use proc_macro2::TokenTree;
use quote::quote;
use quote::ToTokens;
use syn::fold::{fold_expr, Fold};
use syn::parse::Parser;
use syn::parse_macro_input;
use syn::visit::{self, Visit};
use syn::LitInt;
use syn::{token, Expr, ExprParen};
use syn::{File, ItemFn};

#[proc_macro]
pub fn seq(input: TokenStream) -> TokenStream {
    parse_macro_input!(input as Seq).build_macro()
}

#[derive(Debug)]
struct Seq {
    ident: syn::Ident,
    range: std::ops::Range<usize>,
    // The only vilable way is to do it with TokenStream, because
    // since the body of the macro may contain any rust code,
    // it will be easier to walk the tree of tokens in most generic form,
    // which is TokenStream of Groups, Idents, Puncts, and Literals.
    // The alternative is to use either of syn::{visit, visit_mut, fold}, however
    // in order to find and replace Seq.ident with values from Seq.range,
    // I'd have to parse every possible syn token, which is a lot of work
    content: proc_macro2::TokenStream,
}

impl syn::parse::Parse for Seq {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Seq> {
        let content;
        let ident = input.parse()?;
        let _in_token: syn::Token![in] = input.parse()?;
        let range: syn::ExprRange = input.parse()?;
        let _braces = syn::braced!(content in input);
        Ok(Seq {
            ident,
            range: range.into_rust(),
            content: content.parse()?,
        })
    }
}
trait IntoRustRange<Idx> {
    fn into_rust(self) -> std::ops::Range<Idx>;
}
impl IntoRustRange<usize> for syn::ExprRange {
    fn into_rust(self) -> std::ops::Range<usize> {
        let expr_to_u32 = |e: &syn::Expr| {
            if let syn::Expr::Lit(syn::ExprLit { lit, .. }) = e {
                if let syn::Lit::Int(i) = lit {
                    return i.base10_digits().parse::<usize>().unwrap();
                }
            }
            0 // BUG: unhandled exception
        };
        let is_inclusive = if let syn::RangeLimits::HalfOpen(v) = self.limits {
            0
        } else {
            1
        };
        let from = expr_to_u32(self.from.unwrap().deref()); // BUG: unhandled exception
        let to = expr_to_u32(self.to.unwrap().deref()); // BUG: unhandled exception
        return from..to + is_inclusive;
    }
}
impl Seq {
    fn build_macro(&mut self) -> TokenStream {
        // dbg!(&self.range);
        let repeated_content = self
            .range
            .clone()
            .map(|i| Seq::replace_ident(i, self.content.clone(), &self.ident))
            .collect::<Vec<_>>();
        quote! { #(#repeated_content)* }.into()
    }

    fn replace_ident(
        x: usize,
        content: proc_macro2::TokenStream,
        ident: &syn::Ident,
    ) -> proc_macro2::TokenStream {
        let mut content = Vec::from_iter(content.clone());
        let mut i = 0;
        while i < content.len() {
            let replace = match &content[i] {
                TokenTree::Ident(matched_ident)
                    if matched_ident.to_string() == ident.to_string() =>
                {
                    content[i] = proc_macro2::TokenTree::Literal(
                        proc_macro2::Literal::from_str(&x.clone().to_string()).unwrap(),
                    );
                    i += 1;
                    continue;
                }
                _ => false,
            };

            if let TokenTree::Group(group) = &mut content[i] {
                let original_span = group.span();
                let body = Seq::replace_ident(x, group.stream(), ident);
                *group = proc_macro2::Group::new(group.delimiter(), body);
                group.set_span(original_span);
            }

            i += 1;
        }

        (proc_macro2::TokenStream::from_iter(content).into())
    }
}
