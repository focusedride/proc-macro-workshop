use std::ops::Deref;
use std::str::FromStr;

use proc_macro2::TokenTree;
use quote::quote;

#[proc_macro]
pub fn seq(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    syn::parse_macro_input!(input as Seq).build_macro()
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
    fn build_macro(&mut self) -> proc_macro::TokenStream {
        let mut found_repetition = false;
        let expanded = Seq::expand_repetitions(
            &self.ident,
            &self.range,
            self.content.clone(),
            &mut found_repetition,
        );

        if found_repetition {
            quote! { #expanded }.into()
        } else {
            let repeated_body = Seq::repeat(self.content.clone(), self.range.clone(), &self.ident);
            quote! { #(#repeated_body)* }.into()
        }
    }

    fn repeat(
        body: proc_macro2::TokenStream,
        range: std::ops::Range<usize>,
        ident: &syn::Ident,
    ) -> Vec<proc_macro2::TokenStream> {
        let repeated_body = range
            .clone()
            .map(|i| Seq::replace_ident(i, body.clone(), ident))
            .collect::<Vec<_>>();
        repeated_body
    }

    fn replace_ident(
        x: usize,
        content: proc_macro2::TokenStream,
        ident: &syn::Ident,
    ) -> proc_macro2::TokenStream {
        let mut content = Vec::from_iter(content.clone());
        let mut i = 0;
        while i < content.len() {
            match &content[i] {
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

            if i + 3 <= content.len() {
                match &content[i..i + 3] {
                    [TokenTree::Ident(prefix), TokenTree::Punct(tilde), TokenTree::Ident(matched_ident)]
                        if tilde.as_char() == '~'
                            && matched_ident.to_string() == ident.to_string() =>
                    {
                        let ident =
                            proc_macro2::Ident::new(&format!("{}{}", prefix, x), prefix.span());
                        content.splice(i..i + 3, std::iter::once(TokenTree::Ident(ident)));
                        i += 1;
                        continue;
                    }
                    _ => {
                        3; // ignore
                    }
                };
            }

            if let TokenTree::Group(group) = &mut content[i] {
                let original_span = group.span();
                let body = Seq::replace_ident(x, group.stream(), ident);
                *group = proc_macro2::Group::new(group.delimiter(), body);
                group.set_span(original_span);
            }

            i += 1;
        }

        proc_macro2::TokenStream::from_iter(content).into()
    }

    fn expand_repetitions(
        var: &syn::Ident,
        range: &std::ops::Range<usize>,
        body: proc_macro2::TokenStream,
        found_repetition: &mut bool,
    ) -> proc_macro2::TokenStream {
        fn enter_repetition(tokens: &[proc_macro2::TokenTree]) -> Option<proc_macro2::TokenStream> {
            match &tokens[0] {
                TokenTree::Punct(punct) if punct.as_char() == '#' => {}
                _ => return None,
            }
            match &tokens[2] {
                TokenTree::Punct(punct) if punct.as_char() == '*' => {}
                _ => return None,
            }
            match &tokens[1] {
                TokenTree::Group(group)
                    if group.delimiter() == proc_macro2::Delimiter::Parenthesis =>
                {
                    Some(group.stream())
                }
                _ => None,
            }
        }
        let mut tokens = Vec::from_iter(body);

        // Look for `#(...)*`.
        let mut i = 0;
        while i < tokens.len() {
            if let proc_macro2::TokenTree::Group(group) = &mut tokens[i] {
                let content = Seq::expand_repetitions(var, range, group.stream(), found_repetition);
                let original_span = group.span();
                *group = proc_macro2::Group::new(group.delimiter(), content);
                group.set_span(original_span);
                i += 1;
                continue;
            }
            if i + 3 > tokens.len() {
                i += 1;
                continue;
            }
            let template = match enter_repetition(&tokens[i..i + 3]) {
                Some(template) => template,
                None => {
                    i += 1;
                    continue;
                }
            };
            *found_repetition = true;
            let mut repeated = Vec::new();
            for value in range.clone() {
                repeated.extend(Seq::replace_ident(value, template.clone(), var));
            }
            let repeated_len = repeated.len();
            tokens.splice(i..i + 3, repeated);
            i += repeated_len;
        }

        proc_macro2::TokenStream::from_iter(tokens)
    }
}
