
# Derive-Attribute &emsp; [![Latest Version]][crates.io] [![Documentation]][docs.rs]

[Latest Version]: https://img.shields.io/crates/v/derive-attribute.svg
[crates.io]: https://crates.io/crates/derive-attribute

[Documentation]: https://docs.rs/derive-attribute/badge.svg
[docs.rs]: https://docs.rs/derive-attribute

### **A set of macros to automatically deserialize standard attributes**
- #### Compatible with all major versions of [Syn](https://crates.io/crates/syn)
- #### Supports custom deserialization
- #### Can return multiple errors at once
- #### Allows for flexible attribute syntax

## Syn Compatibility
This crate is meant to be used in conjunction with Syn within a procedural macro crate.<br/>
A major version of Syn can be selected as a feature like: `features = ["syn_2"]`.

Note: A Syn version must be selected


## Flexible Attribute Syntax

#### **Implicit Booleans**
` #[some_attr(is_bool)] ` *can also be written as* ` #[some_attr(is_bool = true)] ` <br/>
#### **Seperated Lists**
` #[some_attr(list(key_a = "value", key_b = 123))] `
<br/>
*can also be written as*
<br/>
` #[some_attr(list(key_a = "value"))] ` <br/>
` #[some_attr(list(key_b = 123))] `


## Multiple Errors
Most macros will only return one attribute error at a time. </br>
This crate's macros can return multiple errors at once resulting in a better developer experience.

## Custom Deserialization
Any type that implements `TryFromMeta` can be used as a valid attribute type. </br>
Although Its recommended that you use `CustomArgFromMeta` instead in order to simplify the implementation.

See [example](#custom-deserialization-1)
<br/>

## Attr Arguments
The `#[attr()]` attribute can be added to the attribute struct or its fields to add additional options.

**The full list of arguments are:**

**name [<span style = "color: lightblue">str</span>]** - Renames the field.

**default [<span style = "color: lightblue">bool/str</span>]** - Uses a default value if the argument isn't found.
<span style = "font-size: 10px"> </span><br/>
If its a boolean, the type's implementation of Default::default will be used. \
If its a string, it must be a path to a function that returns the type. 

# Usage
Our attribute type is declared in a procedural macro crate:
```rust
#[derive(Attribute)]
#[attr(name = "my_attr")] // We set the attribute name to 'my_attr'
struct MyAttribute {      // Note: The attribute name will be the struct name in snake_case by default
    name: String,
    // wrapping a type in an option will make it optional
    list: Option<NestedList>, // deserializes a meta list named list i.e. list(num = 1)

    // booleans are always optional
    is_selected: bool,
}
    #[derive(List)]
    pub struct NestedList {
        num: Option<u8>
    }
```
It can then be used to parse the following attribute using the from_attrs method:

```rust
#[my_attr(name = "some_name", is_selected)]
```
<br/>

***lets look at the same attribute used in a derive macro***

## Basic derive

procedural macro crate:
```rust

use derive_attribute::{Attribute, List};
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};


#[derive(Attribute)]
#[attr(name = "my_attr")]
struct MyAttribute {
    name: String,
    // wrapping a type in an option will make it optional
    // deserializes a meta list named list i.e. list(num = 1) 
    list: Option<NestedList>,
    // booleans are always optional
    is_selected: bool,
}
    #[derive(List)]
    pub struct NestedList {
        num: Option<u8>
    }


#[proc_macro_derive(YOUR_MACRO_NAME, attributes(my_attr))]
pub fn derive_my_trait(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = parse_macro_input!(tokens as DeriveInput);

    fn attempt_derive(ast: DeriveInput) -> Result<TokenStream2, Vec<syn::Error>> {
        // Wrapping an attribute in an option makes it optional
        // A missing error won't be returnwd
        let maybe_attribute = <Option<MyAttribute>>::from_attrs(ast.ident.span(), &ast.attrs)?;

        let output: TokenStream2 = {
            // Your Macro Generation Code
        };

        Ok(output)
    }


    let generated_tokens = 
        match attempt_derive(ast) {
            Ok(tokens) => tokens,
            Err(errors) => {
                let compile_errors = errors.into_iter().map(|e| e.to_compile_error());
                quote!(#(#compile_errors)*)
            }
        };

    generated_tokens.into()
}
```

Another crate using our macro

```rust
#[derive(YOUR_MACRO_NAME)]
#[my_attr(name = "some_name", is_selected)]
struct SomeStruct;
```

<br/>

***Now lets add our own argument type***

## Custom Deserialization

proc-macro crate:
```rust
use derive_attribute::{CustomArg, CustomArgFromMeta};

struct ErrorType {
    Warning,
    Severe
}

// Any type that implements 'TryFromMeta' can be deserialized however its a bit verbose
// In order to simplify the implementation we can implement 'CustomArgFromMeta' instead and wrap our type in the 'CustomArg' struct
impl<V: SynVersion> CustomArgFromMeta<V> for ErrorType {
    fn try_from_meta(meta: Self::Metadata) -> Result<Self, ErrorMsg> {
        let maybe_error_kind = 
            match V::deserialize_string(meta) {
                Some(string) => {
                    match string.to_string().as_str() {
                        "warning" => Some(Self::Warning),
                        "severe" => Some(Self::Severe),
                        _ => None
                    }
                }
                None => None
            };

        match maybe_error_kind {
            Some(error_kind) => Ok(error_kind),
            None => Err(InvalidType { expected: r#" "warning" or "severe" "# })
        }
    }
}

```

Our attribute struct now looks like this: 
```rust
#[derive(Attribute)]
#[attr(name = "my_attr")]
struct MyAttribute {
    // In order to use the simplified trait(CustomArgFromMeta) we need to wrap our struct in 'CustomArg'
    error_type: CustomArg<ErrorType>,

    name: String,
    list: Option<u32>,
    is_selected: bool,
}
```
Another crate using our macro:
```rust
#[derive(YOUR_MACRO_NAME)]
#[error(error_type = "warning", name = "some_name", is_selected)]
struct Test;
```



