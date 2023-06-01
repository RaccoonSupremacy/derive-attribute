
use std::{str::FromStr, fmt::Display};

use proc_macro2::{Span, Ident};
use syn_v2::{Attribute, Meta, MetaNameValue, Expr, ExprLit, Lit, punctuated::Punctuated, token::Eq, Token, spanned::Spanned, Path, ExprArray, PathSegment};

use crate::{shared::{SynVersion, GetSpan}};

/// Deserialization functions & types for Syn version 1
pub struct Syn2;

impl SynVersion for Syn2 {
    type Attribute = Attribute;

    type ArgMeta = Meta;

    fn deserialize_key(meta: &Self::ArgMeta) -> Option<String> {
        meta.path().get_ident().map(|id| id.to_string())
    }
    fn deserialize_attr_key(meta: &Self::Attribute) -> Option<String> {
        meta.path().get_ident().map(|id| id.to_string())
    }

    fn deserialize_integer<T>(meta: &Self::ArgMeta) -> Option<T> 
    where
        T: FromStr,
        T::Err: Display
    {
        match meta {
            Meta::NameValue(MetaNameValue { value: Expr::Lit(ExprLit { lit: Lit::Int(literal), .. }), .. }) => {
                literal.base10_parse().map_or(None, Some)
            },
            _ => None
        }
    }
    
    fn deserialize_float<T>(meta: &Self::ArgMeta) ->  Option<T> where T: FromStr, T::Err: Display {
        match meta {
            Meta::NameValue(MetaNameValue { value: Expr::Lit(ExprLit { lit: Lit::Float(literal), .. }), .. }) => {
                literal.base10_parse().map_or(None, Some)
            },
            _ => None
        }
    }

    fn deserialize_string(meta: &Self::ArgMeta) -> Option<String> {
        match meta {
            Meta::NameValue(MetaNameValue { value: Expr::Lit(ExprLit { lit: Lit::Str(literal), .. } ), .. }) => Some(literal.value()),
            _ => None
        }
    }
    fn deserialize_bool(meta: &Self::ArgMeta) -> Option<bool> {
        match meta {
            Meta::Path(_) => Some(true),
            Meta::NameValue(MetaNameValue { value: Expr::Lit( ExprLit { lit: Lit::Bool(literal), .. } ), .. }) => Some(literal.value()),
            _ => None
        }
    }

    fn deserialize_list_args(meta: &Self::ArgMeta) -> Option<Vec<Self::ArgMeta>> {
        match meta {
            Meta::List(list) => {
                let x = list.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated);
                match x {
                    Ok(x) => Some(x.into_iter().collect()),
                    Err(_) => None
                }
            },
            _ => None
        }
    }
    fn deserialize_attr_args(attr: &Self::Attribute) -> Option<Vec<Self::ArgMeta>> {
        let maybe_args = attr.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated);
        match maybe_args {
            Ok(args) => Some(args.into_iter().collect()),
            Err(_) => None
        }
    }

    fn deserialize_array(meta: &Self::ArgMeta) -> Option<Vec<Self::ArgMeta>> {
        match meta {
            Meta::NameValue(MetaNameValue { value: Expr::Array(ExprArray { elems, .. }), .. }) => {
                let list = 
                    elems
                        .into_iter()
                        .map(|e| 
                            Meta::NameValue(
                                MetaNameValue { 
                                    path: 
                                        Path { 
                                            leading_colon: None, 
                                            segments: 
                                                vec![PathSegment {
                                                    ident: Ident::new("_", e.span()), 
                                                    arguments: syn_v2::PathArguments::None 
                                                }]
                                                .into_iter().collect() 
                                        }, 
                                    eq_token: Eq { spans: [meta.span()] }, 
                                    value: e.clone() 
                                }
                            )
                        ).collect();
                Some(list)
            }
            _ => None
        }
    }

    type Error = syn_v2::Error;
    fn convert_error(error: crate::shared::Error) -> Self::Error {
        syn_v2::Error::new(error.location, error.msg)
    }
}

impl GetSpan for Attribute {
    fn get_span(&self) -> Span { self.path().span() }
}

impl GetSpan for Meta {
    fn get_span(&self) -> Span { self.path().span() }
}

