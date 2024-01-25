use proc_macro2::TokenStream;
use quote::quote;
use std::collections::HashMap;
use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    token::Comma,
    Data, DeriveInput, Field, Fields, GenericParam, Ident,
};

pub fn derive_from_multipart(mut input: DeriveInput) -> syn::Result<TokenStream> {
    for i in 0..input.generics.params.len() {
        let generic = &mut input.generics.params[i];
        if let GenericParam::Type(ref mut generic_ty) = generic {
            let bound = syn::parse_str("::mincat::extract::form::FromMultipart")?;
            generic_ty.bounds.push(bound);
        }
    }

    let name = input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let fields = match input.data {
        Data::Struct(data) => match data.fields {
            Fields::Named(fields) => fields.named,
            _ => panic!("Only named fields are supported"),
        },
        _ => panic!("Only structs are supported"),
    };

    let field_attrs = get_fields_attributes(&fields)?;

    let field_names = fields
        .iter()
        .map(|f| f.ident.as_ref().unwrap())
        .cloned()
        .collect::<Vec<_>>();

    let mut field_parsers = Vec::new();

    for f in fields {
        let original_name = f.ident.as_ref().unwrap();
        let name_str = original_name.to_string();
        let attr = field_attrs.get(&name_str).cloned();
        let field_name_str = attr
            .clone()
            .and_then(|attr| attr.rename)
            .unwrap_or_else(|| original_name.to_string().clone());

        let field_ty = f.ty;
        let parser = match attr.and_then(|s| s.with) {
            Some(with) => {
                let from_multipart_fn = match syn::parse_str::<syn::Path>(&with) {
                    Ok(p) => p,
                    Err(err) => {
                        return Err(err);
                    }
                };

                quote! { #from_multipart_fn ( multipart, ::mincat::extract::form::FormContext { field_name: Some( #field_name_str ) } )? }
            }
            None => {
                quote! {
                    <#field_ty as ::mincat::extract::form::FromMultipart>::from_multipart(
                        multipart,
                        ::mincat::extract::form::FormContext {
                            field_name: Some( #field_name_str ),
                        },
                    )?
                }
            }
        };

        field_parsers.push(quote! {
            let #original_name = #parser;
        });
    }

    let expanded = quote! {
        #[automatically_derived]
        impl #impl_generics ::mincat::extract::form::FromMultipart for #name #ty_generics #where_clause {
            fn from_multipart<'a>(multipart: &::mincat::extract::form::MultipartForm, _ctx: ::mincat::extract::form::FormContext<'_>) -> Result<Self, ::mincat::extract::form::Error> {
                #(#field_parsers)*

                Ok(Self {
                    #(#field_names),*
                })
            }
        }

        impl #impl_generics ::mincat::extract::form::FromMultipartNull for #name #ty_generics #where_clause {
            fn is_null() -> bool {
                false
            }

            fn generate(_: ::mincat::extract::form::Multipart<'static>) -> Self {
                todo!();
            }
        }
    };

    // return the tokens
    Ok(expanded)
}

#[derive(Debug, Clone)]
struct MulterAttribute {
    // #[multer(rename = "other_name")]
    rename: Option<String>,

    // #[multer(with = "path::to::function")]
    with: Option<String>,
}

impl Parse for MulterAttribute {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut multer_attribute = MulterAttribute {
            rename: None,
            with: None,
        };

        while !input.is_empty() {
            let lookahead = input.lookahead1();

            if input.peek(Ident) {
                let path: syn::Path = input.parse()?;

                // #[multer(rename = "...")]
                if path.is_ident("rename") {
                    let _: syn::Token![=] = input.parse()?;
                    let rename_value: syn::LitStr = input.parse()?;
                    multer_attribute.rename = Some(rename_value.value());
                }
                // #[multer(with = "...")]
                else if path.is_ident("with") {
                    let _: syn::Token![=] = input.parse()?;
                    let with_value: syn::LitStr = input.parse()?;
                    multer_attribute.with = Some(with_value.value());
                } else {
                    return Err(lookahead.error());
                }
            } else {
                return Err(lookahead.error());
            }

            if !input.is_empty() {
                let _: syn::Token![,] = input.parse()?;
            }
        }

        Ok(multer_attribute)
    }
}

fn get_fields_attributes(
    fields: &Punctuated<Field, Comma>,
) -> syn::Result<HashMap<String, MulterAttribute>> {
    let mut attrs = HashMap::new();

    for field in fields {
        for attr in &field.attrs {
            if !attr.path().is_ident("multer") {
                continue;
            }

            let multer_attr: MulterAttribute = attr.parse_args()?;
            let field_name = field.ident.as_ref().unwrap();
            attrs.insert(field_name.to_string(), multer_attr);
        }
    }

    Ok(attrs)
}

pub fn generate(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input: DeriveInput = syn::parse_macro_input!(input as DeriveInput);
    match derive_from_multipart(input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.into_compile_error().into(),
    }
}
