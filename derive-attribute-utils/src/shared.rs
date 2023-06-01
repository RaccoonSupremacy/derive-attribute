use std::{str::FromStr, fmt::Display};

use proc_macro2::Span;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ErrorMsg {
    #[error("Failed to Parse Attribute: expected list of key/value pairs EX) ATTR_NAME(x = 1) and/or booleans EX) ATTR_NAME(is_x)")]
    FailedToParseAttr,

    #[error("Invalid Item: expected struct")]
    InvalidItem,

    #[error("Missing Attribute: attribute '{0}' is required")]
    MissingAttribute(&'static str),

    #[error("Missing Argument: '{0}' is required")]
    MissingArg(&'static str),
    #[error("Invalid Type: expected {expected}")]
    InvalidType{expected: &'static str},
    #[error("Duplicate Argument")]
    DuplicateArg,
    #[error("Invalid Argument")]
    InvalidArg,
}
use ErrorMsg::*;


#[derive(Debug)]
pub struct Error {
    pub msg: ErrorMsg,
    pub location: Span,
}
impl Error {
    pub fn new(location: Span, msg: ErrorMsg) -> Self {
        Self {
            location,
            msg
        }
    }
}

/// Combines an argument with a previously stored instance.
pub trait Concat: Sized {
    /// Determines whether a duplicate is allowed or if an error should be thrown. \
    /// This logic is handled in the implementation for `ArgResult`. \
    /// Typically only true for primitive types.
    const NO_DUPLICATES: bool = true;

    /// Combines an argument.
    fn concat(&mut self, other: Self) { *self = other }
}
impl<T: Concat> Concat for ArgResult<T> {
    fn concat(&mut self, other: Self) {
        let mut other = other;
        if other.is_found() {
            self.location = other.location;
            if self.is_found() && T::NO_DUPLICATES {
                self.add_error(DuplicateArg);
            }
        }
        if other.found_with_errors() {
            self.errors.append(&mut other.errors)
        }
        if other.found_with_value() {    
            match self.value {
                Some(ref mut value) => {
                    match other.value {
                        Some(other_value) => {
                            value.concat(other_value);
                        },
                        None => {}
                    }
    
                }
                None => self.value = other.value
            }
        }
    }
}




/// A result type that can contain multiple errors and a value at the same time.
#[derive(Debug)]
pub struct ArgResult<T> {
    pub value: Option<T>,
    pub errors: Vec<Error>,
    pub location: Span,
}
impl<T> ArgResult<T> {
    pub fn new(location: Span) -> Self {
        Self {
            value: None,
            errors: vec![],
            location,
        }
    }
    /// Adds an error using the stored Span.
    pub fn add_error(&mut self, msg: ErrorMsg) {
        self.errors.push(Error::new(self.location, msg));
    }

    /// Adds a value.
    pub fn add_value(&mut self, value: T) {
        self.value = Some(value);
    }

    /// Adds a value or an error that uses the stored Span depending on the result variant.
    pub fn add_result(&mut self, value: Result<T, ErrorMsg>) {
        match value {
            Ok(value) => self.add_value(value),
            Err(error) => self.add_error(error)
        }
    }
    
    pub fn is_found(&self) -> bool { self.errors.len() > 0 || self.value.is_some() }
    pub fn found_with_errors(&self) -> bool { self.errors.len() > 0 }
    pub fn found_with_value(&self) -> bool { self.value.is_some() }
}


/// Represents a Syn version and how it can parse attribute data into values
pub trait SynVersion: Sized {
    /// A type that represents an attribute.
    type Attribute: GetSpan;

    /// Parses an attribute list into a vector of its elements as metadata.
    fn deserialize_attr_args(attr: &Self::Attribute) -> Option<Vec<Self::ArgMeta>>;
    /// Parses a nested list into a vector of its elements as metadata.
    fn deserialize_list_args(meta: &Self::ArgMeta) -> Option<Vec<Self::ArgMeta>>;

    /// Metadata that can be used to deserialize a value.
    type ArgMeta: GetSpan;

    /// Gets the key from a key value pair as a string.
    fn deserialize_key(meta: &Self::ArgMeta) -> Option<String>;
    
    /// Gets the name of an attribute list.
    fn deserialize_attr_key(meta: &Self::Attribute) -> Option<String>;

    /// Attempts to get an integer from an argument. Returns None if the argument is a different type.
    fn deserialize_integer<T>(meta: &Self::ArgMeta) -> Option<T> where T: FromStr, T::Err: Display;

    /// Attempts to get a float from an argument. Returns None if the argument is a different type.
    fn deserialize_float<T>(meta: &Self::ArgMeta) ->  Option<T> where T: FromStr, T::Err: Display;

    /// Attempts to get a string from an argument. Returns None if the argument is a different type.
    fn deserialize_string(meta: &Self::ArgMeta) -> Option<String>;

    /// Attempts to get a boolean from an argument. Returns None if the argument is a different type.
    fn deserialize_bool(meta: &Self::ArgMeta) -> Option<bool>;

    /// Attempts to get an array from an argument and returns a vector of its elements as metadata.
    fn deserialize_array(meta: &Self::ArgMeta) -> Option<Vec<Self::ArgMeta>>;

    /// A Syn Error.
    type Error;

    /// Converts this crates error into a Syn error.
    fn convert_error(error: Error) -> Self::Error;
}

/// Gets the Span of Syn metadata
pub trait GetSpan {
    fn get_span(&self) -> Span;
}


/// A trait for deserializing Syn metadata. \
/// Its recommended that you use the `CustomArgFromMeta` trait for deserializing simple arguments.
pub trait TryFromMeta<V: SynVersion>: Sized {
    /// This is the initial type and will be converted to `Self` in the `validate` function. \
    /// For most types this is typically Self but for lists this is a builder.
    type InitialType: Concat;

    /// The metadata of an argument or an attribute.
    type Metadata;
    /// Looks for values & errors in the metadata. \
    /// The returned result is added to the argument's state using the 
    /// `concat` method on `ArgResult` which in turn will call the initial type's `Concat` implementation if found.
    fn try_from_meta(meta: Self::Metadata) -> ArgResult<Self::InitialType>;
    /// Converts the initial type to Self or returns found errors. \
    /// If the argument is required it should return a missing error. \
    /// Wrapper types such as Option can overwrite this.
    fn validate(state: ArgResult<Self::InitialType>, arg_name: &'static str) -> Result<Self, Vec<Error>>;
}

/// Validates a simple required type.
pub fn required_validation<A, B: From<A>>(state: ArgResult<A>, arg_name: &'static str) -> Result<B, Vec<Error>> {
    let mut state = state;
    match state.found_with_errors() {
        true => Err(state.errors),
        false if state.value.is_none() => {
            state.add_error(MissingArg(arg_name));
            Err(state.errors)
        }
        false => Ok(state.value.unwrap().into())
    }
}

impl Concat for String {}
impl<V: SynVersion> TryFromMeta<V> for String {
    type InitialType = Self;

    type Metadata = V::ArgMeta;
    fn try_from_meta(meta: Self::Metadata) -> ArgResult<Self> {
        let mut result = ArgResult::new(meta.get_span());

        let maybe_string = V::deserialize_string(&meta);

        match maybe_string {
            Some(string) => result.add_value(string),
            None => result.add_error(InvalidType { expected: "string" })
        }

        result
    }

    fn validate(state: ArgResult<Self::InitialType>, arg_name: &'static str) -> Result<Self, Vec<Error>> {
        required_validation(state, arg_name)
    }
}


impl Concat for bool {}
impl<V: SynVersion> TryFromMeta<V> for bool {
    type InitialType = Self;

    type Metadata = V::ArgMeta;
    fn try_from_meta(meta: Self::Metadata) -> ArgResult<Self::InitialType> {
        let mut result = ArgResult::new(meta.get_span());

        let maybe_bool = V::deserialize_bool(&meta);

        match maybe_bool {
            Some(value) => result.add_value(value),
            None => result.add_error(InvalidType { expected: "boolean" })
        }

        result
    }

    fn validate(state: ArgResult<Self::InitialType>, _arg_name: &'static str) -> Result<Self, Vec<Error>> {
        match state.found_with_errors() {
            true => Err(state.errors),
            false if state.value.is_none() => Ok(false),
            false => Ok(state.value.unwrap())
        }
    }
}

impl<T: Concat> Concat for Vec<T> {
    const NO_DUPLICATES: bool = false;
    fn concat(&mut self, other: Self) {
        let mut other = other;
        self.append(&mut other);
    }
}
impl<V: SynVersion, T: TryFromMeta<V, Metadata = V::ArgMeta>> TryFromMeta<V> for Vec<T> {
    type InitialType = Vec<ArgResult<T::InitialType>>;
    type Metadata = V::ArgMeta;

    fn try_from_meta(meta: Self::Metadata) -> ArgResult<Self::InitialType> {
        let mut result = ArgResult::new(meta.get_span());
        let array = 
            match V::deserialize_array(&meta) {
                Some(array) => array,
                None => {
                    result.add_error(InvalidType { expected: "array" });
                    return result;
                }
            };

        let mut values = vec![];
        for meta in array {
            let element = T::try_from_meta(meta);
            values.push(element);
        }

        result.add_value(values);

        result
    }

    fn validate(state: ArgResult<Self::InitialType>, arg_name: &'static str) -> Result<Self, Vec<Error>> {
        let mut state = state;

        let values = 
            match state.found_with_errors() {
                true => return Err(state.errors),
                false if state.value.is_none() => {
                    state.add_error(MissingArg(arg_name));
                    return Err(state.errors);
                },
                false => state.value.unwrap()
            };

        let mut y = vec![];
        for element in values {
            let x = 
                match T::validate(element, arg_name) {
                    Ok(val) => val,
                    Err(ref mut errors) => {
                        state.errors.append(errors);
                        continue;
                    }
                };

            y.push(x);
        }

        match state.errors.len() {
            0 => Ok(y),
            _ => Err(state.errors)
        }
    }
}


impl<V: SynVersion, T: TryFromMeta<V>> TryFromMeta<V> for Option<T> {
    type InitialType = T::InitialType;

    type Metadata = T::Metadata;
    fn try_from_meta(meta: Self::Metadata) -> ArgResult<Self::InitialType> {
        T::try_from_meta(meta)
    }

    fn validate(state: ArgResult<Self::InitialType>, arg_name: &'static str) -> Result<Self, Vec<Error>> {
        match state.value {
            Some(_) => Ok(Some(T::validate(state, arg_name)?)),
            None if state.found_with_errors() => Err(state.errors),
            None => Ok(None)
        }
    }
}


pub trait AttributeName {
    const NAME: &'static str;
}

impl<T: AttributeName> AttributeName for Option<T> {
    const NAME: &'static str = T::NAME;
}

impl<V: SynVersion, T: Attribute<V>> Attribute<V> for Option<T>
where 
    Self: TryFromMeta<V, Metadata = V::Attribute>
{}



/// Represents a struct that can be deserialized from Syn attributes.
pub trait Attribute<V: SynVersion>: AttributeName + TryFromMeta<V, Metadata = V::Attribute> {
    /// Creates a deserialized attribute from a list of Syn attributes.
    fn from_attrs(location: Span, attrs: Vec<V::Attribute>) -> Result<Self, Vec<V::Error>> {
        let mut result = ArgResult::new(location);

        for attr in attrs {
            let maybe_key = V::deserialize_attr_key(&attr);
            let found_attribute = matches!(maybe_key, Some(key) if key == Self::NAME);
            if found_attribute == false { continue; }
            

            let attr = Self::try_from_meta(attr);
            result.concat(attr);
        }

        let maybe_attr = <Self as TryFromMeta<V>>::validate(result, Self::NAME);

        maybe_attr.map_err(|e| e.into_iter().map(|e| V::convert_error(e)).collect())
    }
}


/// A simplified version of the `TryFromMeta` trait. Types that implement this must be wrapped in the `CustomArg` struct. 
pub trait CustomArgFromMeta<V: SynVersion>: Sized {
    fn try_from_meta(meta: V::ArgMeta) -> Result<Self, ErrorMsg>;
}

/// Allows a type to implement `CustomArgFromMeta`, a simplified version of `TryFromMeta`.
#[derive(Debug)]
pub struct CustomArg<T>(pub T);
impl<V: SynVersion, T: CustomArgFromMeta<V>> TryFromMeta<V> for CustomArg<T> {
    type InitialType = Self;
    type Metadata = V::ArgMeta;
    
    fn try_from_meta(meta: Self::Metadata) -> ArgResult<Self::InitialType> {
        let mut result = ArgResult::new(meta.get_span());
        let x = <T as CustomArgFromMeta<V>>::try_from_meta(meta);

        result.add_result(x);
        
        let v = result.value.map(|v| Self(v));
        ArgResult { value: v, errors: result.errors, location: result.location }


    }

    fn validate(state: ArgResult<Self::InitialType>, arg_name: &'static str) -> Result<Self, Vec<Error>> {
        required_validation(state, arg_name)
    }
}
impl<T> Concat for CustomArg<T> {}
impl<T: Default> Default for CustomArg<T> {
    fn default() -> Self { Self(T::default()) }
}




macro_rules! impl_integer {
    ($($type_name: ident), *) => {
        $(
            impl Concat for $type_name {}
            impl<V: SynVersion> TryFromMeta<V> for $type_name {
                type InitialType = Self;
                type Metadata = V::ArgMeta;
                fn try_from_meta(meta: Self::Metadata) -> ArgResult<Self::InitialType> {
                    let mut result = ArgResult::new(meta.get_span());

                    let maybe_int = V::deserialize_integer(&meta);

                    match maybe_int {
                        Some(value) => result.add_value(value),
                        None => result.add_error(InvalidType { expected: stringify!($type_name) })
                    }

                    result
                }

                fn validate(state: ArgResult<Self::InitialType>, arg_name: &'static str) -> Result<Self, Vec<Error>> {
                    required_validation(state, arg_name)
                }
            }

        )*
    };
}
macro_rules! impl_float {
    ($($type_name: ident), *) => {
        $(
            impl Concat for $type_name {}
            impl<V: SynVersion> TryFromMeta<V> for $type_name {
                type InitialType = Self;
                type Metadata = V::ArgMeta;
                fn try_from_meta(meta: Self::Metadata) -> ArgResult<Self::InitialType> {
                    let mut result = ArgResult::new(meta.get_span());

                    let maybe_int = V::deserialize_integer(&meta);

                    match maybe_int {
                        Some(value) => result.add_value(value),
                        None => result.add_error(InvalidType { expected: stringify!($type_name) })
                    }

                    result
                }

                fn validate(state: ArgResult<Self::InitialType>, arg_name: &'static str) -> Result<Self, Vec<Error>> {
                    required_validation(state, arg_name)
                }
            }
        )*
    };
}

impl_integer!(u8, u16, u32, u64, u128, i8, i16, i32, i64, i128);
impl_float!(f32, f64);
