use quote::{quote, ToTokens};

#[proc_macro_derive(CustomDebug, attributes(debug))]
pub fn derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(input as syn::ItemStruct);
    let i = &input.ident;
    let u = syn::LitStr::new(
        input.ident.to_string().as_str(),
        proc_macro2::Span::call_site(),
    );
    let b = input.fields.build_debug_struct_fields();
    let fnfmt = quote! {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct(#u) #b .finish()
        }
    };
    if let Some(g) = input.generics.type_params().last() {
        let ident = &g.ident;
        if let Some(attr) = input.attrs.first() {
            if let Ok(syn::Meta::List(syn::MetaList { nested, .. })) = attr.parse_meta() {
                if let Some(syn::NestedMeta::Meta(syn::Meta::NameValue(syn::MetaNameValue {
                    path,
                    lit,
                    ..
                }))) = nested.first()
                {
                    if path.is_ident("bound") {
                        if let syn::Lit::Str(l) = lit {
                            if let Ok(syn::WherePredicate::Type(syn::PredicateType {
                                bounded_ty,
                                bounds,
                                ..
                            })) = syn::parse_str::<syn::WherePredicate>(&l.value())
                            {
                                return proc_macro::TokenStream::from(quote! {
                                    // implementing `T: Trait` is trivial at this point...
                                    impl<T: Trait> Debug for Wrapper<T>
                                    where
                                        #bounded_ty: #bounds,
                                    { #fnfmt }
                                });
                            }
                        };
                    }
                }
            }
            proc_macro::TokenStream::from(quote! {
                impl<T: Trait> Debug for Wrapper<T>
                where
                    T::Value: Debug,
                { #fnfmt }
            })
        } else if let Some(_q) = input.fields.get_phantom_field() {
            proc_macro::TokenStream::from(quote! {
                impl<#ident> std::fmt::Debug for #i<#ident>
                where
                    PhantomData<#ident>: Debug
                { #fnfmt }
            })
        } else if let Some(syn::TypeParamBound::Trait(syn::TraitBound { path, .. })) =
            g.bounds.first()
        {
            if let Some(syn::PathSegment {
                ident: assoctrait, ..
            }) = path.segments.first()
            {
                let assoctype = input.fields.get_associated_type().unwrap();
                proc_macro::TokenStream::from(quote! {
                    impl<#ident: #assoctrait> Debug for #i<#ident>
                    where
                        #assoctype: std::fmt::Debug
                    { #fnfmt }
                })
            } else {
                proc_macro::TokenStream::from(quote! {})
            }
        } else {
            proc_macro::TokenStream::from(quote! {
                impl<#ident: std::fmt::Debug> std::fmt::Debug for #i<#ident> { #fnfmt }
            })
        }
    } else {
        proc_macro::TokenStream::from(quote! {
            impl std::fmt::Debug for #i { #fnfmt }
        })
    }
}

trait FieldsParser {
    fn get_phantom_field(&self) -> std::option::Option<syn::Ident>;
    fn get_associated_type(&self) -> std::option::Option<proc_macro2::TokenStream>;
    fn build_debug_struct_fields(&self) -> proc_macro2::TokenStream;
    fn debug_field_format(f: &syn::Field) -> syn::Result<proc_macro2::TokenStream>;
}
impl FieldsParser for syn::Fields {
    fn get_phantom_field(&self) -> Option<syn::Ident> {
        for f in self.iter() {
            if let syn::Type::Path(syn::TypePath { path, .. }) = &f.ty {
                if let Some(s) = path.segments.first() {
                    if s.ident.to_string().as_str() == "PhantomData" {
                        return f.ident.clone();
                    }
                }
            }
        }
        None
    }
    fn get_associated_type(&self) -> Option<proc_macro2::TokenStream> {
        for f in self.iter() {
            if let syn::Type::Path(syn::TypePath { path, .. }) = &f.ty {
                if let Some(syn::PathSegment { arguments, .. }) = path.segments.first() {
                    if let syn::PathArguments::AngleBracketed(
                        syn::AngleBracketedGenericArguments { args, .. },
                    ) = arguments
                    {
                        if let Some(syn::GenericArgument::Type(t)) = args.first() {
                            return Some(t.to_token_stream());
                        }
                    }
                }
            }
        }
        None
    }
    fn build_debug_struct_fields(&self) -> proc_macro2::TokenStream {
        let fields = self
            .iter()
            .map(|f| {
                let a = syn::LitStr::new(
                    f.ident.as_ref().unwrap().to_string().as_str(),
                    proc_macro2::Span::mixed_site(),
                );
                let c = Self::debug_field_format(&f).unwrap();
                let r = quote! { .field(#a, #c) };
                return r;
            })
            .collect::<Vec<_>>();
        quote! {#(#fields)*}.into()
    }
    fn debug_field_format(f: &syn::Field) -> syn::Result<proc_macro2::TokenStream> {
        let field_name = &f.ident;
        if let Some(a) = f.attrs.first() {
            if let Ok(syn::Meta::NameValue(syn::MetaNameValue { lit, path, .. })) = a.parse_meta() {
                if path.is_ident("debug") {
                    if let syn::Lit::Str(l) = lit {
                        return Ok(quote! { &format_args!(#l, &self.#field_name)});
                    }
                } else {
                    return Err(syn::Error::new_spanned(f, "only debug is implpemented"));
                }
            }
        }
        Ok(quote! { &self.#field_name })
    }
}
