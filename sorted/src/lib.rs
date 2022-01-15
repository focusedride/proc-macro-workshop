#![allow(unused_imports, unused_must_use, dead_code, unused_parens)]
use std::vec;

use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;
use syn::visit_mut::{self, VisitMut};
use syn::{parse_quote, Expr, File, Lit, LitInt};

#[proc_macro_attribute]
pub fn sorted(_args: TokenStream, input: TokenStream) -> TokenStream {
    let mut out = input.clone();
    let input = parse_macro_input!(input as syn::Item);
    // let args = parse_macro_input!(args as syn::Item);
    if let Err(e) = sorted_variants(input) {
        out.extend(TokenStream::from(e.to_compile_error()));
    }
    out
}

fn sorted_variants(input: syn::Item) -> syn::Result<proc_macro2::TokenStream> {
    if let syn::Item::Enum(x) = input {
        // dbg!(&x);
        if let Some(out_of_order) = x
            .variants
            .iter()
            .collect::<Vec<&syn::Variant>>()
            .windows(2)
            .map(|v| {
                if v[0].ident.to_string() <= v[1].ident.to_string() {
                    None
                } else {
                    Some(v[1])
                }
            })
            .filter(|v| v.is_some())
            .map(|v| v.unwrap())
            .inspect(|x| println!("{:?}", x.ident.to_string()))
            .collect::<Vec<&syn::Variant>>()
            .first()
        {
            let before = x
                .variants
                .iter()
                .find(|x| x.ident.to_string() >= out_of_order.ident.to_string())
                .unwrap();
            return Err(syn::Error::new(
                out_of_order.ident.span(),
                format!(
                    "{} should sort before {}",
                    out_of_order.ident.to_string(),
                    before.ident.to_string()
                ),
            ));
        }

        Ok(quote! {})
    } else {
        Err(syn::Error::new(
            proc_macro2::Span::call_site(),
            "expected enum or match expression",
        ))
    }
}

#[proc_macro_attribute]
pub fn check(args: TokenStream, input: TokenStream) -> TokenStream {
    let mut f = parse_macro_input!(input as syn::ItemFn);
    assert!(args.is_empty());
    let mut l = LexiographicMatching::default();
    l.visit_item_fn_mut(&mut f);
    let mut ts = quote! {#f};
    ts.extend(l.errors.into_iter().take(1).map(|e| e.to_compile_error()));
    ts.into()
}

#[derive(Default, Debug)]
struct LexiographicMatching {
    errors: Vec<syn::Error>,
}

impl VisitMut for LexiographicMatching {
    fn visit_expr_match_mut(&mut self, i: &mut syn::ExprMatch) {
        let path_as_string = |path: &syn::Path| {
            path.segments
                .clone()
                .iter()
                .map(|p| format!("{}", quote! {#p}))
                .collect::<Vec<String>>()
                .join("::")
        };

        let extract_match_arm_patterns = |syn::Arm { pat, .. }| match pat {
            syn::Pat::Ident(syn::PatIdent { ident: ref id, .. }) => Some(id.clone().into()),
            syn::Pat::Path(ref p) => Some(p.path.clone()),
            syn::Pat::Struct(p) => Some(p.path.clone()),
            syn::Pat::TupleStruct(p) => Some(p.path.clone()),
            _ => None,
        };

        let mut arm_patterns: Vec<String> = vec![];
        let mut counter = 0;
        if i.attrs.iter().any(|a| a.path.is_ident("sorted")) {
            i.attrs.retain(|a| !a.path.is_ident("sorted"));
            for arm in &i.arms {
                counter += 1;
                let path = if let Some(path) = extract_match_arm_patterns(arm.clone()) {
                    path
                } else if let syn::Pat::Wild(x) = &arm.pat {
                    if counter != i.arms.len() {
                        self.errors
                            .push(syn::Error::new_spanned(&x, "unsupported by #[sorted]"));
                    }
                    continue;
                } else {
                    self.errors.push(syn::Error::new_spanned(
                        &arm.pat,
                        "unsupported by #[sorted]",
                    ));
                    continue;
                };
                let name = path_as_string(&path);
                if arm_patterns
                    .last()
                    .map(|last| &name < last)
                    .unwrap_or(false)
                {
                    let next_lex_i = arm_patterns.binary_search(&name).unwrap_err();
                    self.errors.push(syn::Error::new_spanned(
                        path,
                        format!("{} should sort before {}", name, arm_patterns[next_lex_i]),
                    ));
                }
                arm_patterns.push(name);
            }
        }

        // Delegate to the default impl to visit nested expressions.
        visit_mut::visit_expr_match_mut(self, i);
    }
}
