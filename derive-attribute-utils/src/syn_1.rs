use std::{str::FromStr, fmt::Display};

use proc_macro2::Ident;
use syn_v1::{NestedMeta, spanned::Spanned, Attribute, Meta, MetaList, MetaNameValue, Lit, parse_quote, ExprArray, Expr, PathSegment, PathArguments, Path, token::Eq};

use crate::{shared::GetSpan, SynVersion};

/// Deserialization functions & types for Syn version 1
pub struct Syn1;

impl SynVersion for Syn1 {
    type Attribute = Attribute;
    type ArgMeta = NestedMeta;

    fn deserialize_attr_args(attr: &Self::Attribute) -> Option<Vec<Self::ArgMeta>> {
        match attr.parse_meta() {
            Ok(Meta::List(MetaList { nested, .. })) => Some(nested.into_iter().collect()),
            _ => None
        }
    }

    fn deserialize_list_args(meta: &Self::ArgMeta) -> Option<Vec<Self::ArgMeta>> {
        match meta.clone() {
            NestedMeta::Meta(Meta::List(MetaList { nested, .. })) => Some(nested.into_iter().collect()),
            _ => None
        }
    }

    fn deserialize_bool(meta: &Self::ArgMeta) -> Option<bool> {
        match meta {
            NestedMeta::Meta(Meta::NameValue(MetaNameValue { lit: Lit::Bool(literal), .. })) => Some(literal.value()),
            NestedMeta::Meta(Meta::Path(path)) => Some(true),
            _ => None
        }
    }

    fn deserialize_attr_key(meta: &Self::Attribute) -> Option<String> {
        meta.path.get_ident().map(|id| id.to_string())
    }

    fn deserialize_key(meta: &Self::ArgMeta) -> Option<String> {
        match meta {
            NestedMeta::Meta(Meta::NameValue(MetaNameValue { path, .. })) => path.get_ident().map(|id| id.to_string()),
            _ => None
        }
    }

    fn deserialize_integer<T>(meta: &Self::ArgMeta) -> Option<T> where T: std::str::FromStr, T::Err: std::fmt::Display {
        match meta {
            NestedMeta::Meta(Meta::NameValue(MetaNameValue { lit: Lit::Int(literal), .. })) => literal.base10_parse().map_or(None, Some),
            _ => None
        }
    }

    fn deserialize_float<T>(meta: &Self::ArgMeta) ->  Option<T> where T: FromStr, T::Err: Display {
        match meta {
            NestedMeta::Meta(Meta::NameValue(MetaNameValue { lit: Lit::Float(literal), .. })) => literal.base10_parse().map_or(None, Some),
            _ => None
        }
    }

    fn deserialize_string(meta: &Self::ArgMeta) -> Option<String> {
        match meta { 
            NestedMeta::Meta(Meta::NameValue(MetaNameValue { lit: Lit::Str(literal), .. })) => Some(literal.value()),
            _ => None
        }
    }

    fn deserialize_array(meta: &Self::ArgMeta) -> Option<Vec<Self::ArgMeta>> {
        unimplemented!("Parsing arrays/vectors is not implemented for Syn 1");
    }

    type Error = syn_v1::Error;
    fn convert_error(error: crate::Error) -> Self::Error {
        syn_v1::Error::new(error.location, error.msg)
    }
}


impl GetSpan for Attribute {
    fn get_span(&self) -> proc_macro2::Span {
        self.span()
    }
}
impl GetSpan for NestedMeta {
    fn get_span(&self) -> proc_macro2::Span {
        self.span()
    }
}