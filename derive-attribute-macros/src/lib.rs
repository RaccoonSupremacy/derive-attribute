use convert_case::{Casing, Case};
use proc_macro2::{Span, Ident, TokenStream};
use quote::{quote, TokenStreamExt, format_ident};
use syn_v2::{DeriveInput, Data, spanned::Spanned, Field, parse_macro_input};

use derive_attribute_utils::{TryFromMeta, Syn2, ArgResult, Error, ErrorMsg::{*, self}, SynVersion, Concat, GetSpan, AttributeName, Attribute, CustomArgFromMeta, CustomArg};

#[proc_macro_derive(Attribute, attributes(attr))]
pub fn derive_attribute(input: proc_macro::TokenStream) -> proc_macro::TokenStream {

    let ast = parse_macro_input!(input as DeriveInput);

    let maybe_output = attempt_derive_attr(ast);
    let output = 
        match maybe_output {
            Ok(output) => output,
            Err(errors) => {
                let compile_errors = errors.into_iter().map(|e| e.to_compile_error());
                quote!(#(#compile_errors)*)
            }
        };

    output.into()
}
fn attempt_derive_attr(ast: DeriveInput) -> Result<TokenStream, Vec<syn_v2::Error>> {
    let mut all_errors = vec![];

    let maybe_container_attr = AttributeAttribute::from_attrs(ast.ident.span(), ast.attrs)?;

    let struct_data =
        match ast.data {
            Data::Struct(struct_date) => struct_date,
            _ => {
                all_errors.push(syn_v2::Error::new(ast.ident.span(), "Invalid body expected struct"));
                return Err(all_errors)
            }
        };

    let mut builder = AttributeTraitBuilder::new(ast.ident, maybe_container_attr);
    


    for field in struct_data.fields {
        let field_attr = 
            match AttributeAttribute::from_attrs(field.ident.span(), field.attrs.clone()) {
                Ok(attr) => attr,
                Err(ref mut errors) => {
                    all_errors.append(errors);
                    continue;
                }
            };

        builder.check_field(field, field_attr);
        
    }

    let output = builder.build();

    match all_errors.len() {
        0 => Ok(output),
        _ => Err(all_errors)
    }
}



#[proc_macro_derive(List, attributes(attr))]
pub fn derive_list(input: proc_macro::TokenStream) -> proc_macro::TokenStream {

    let ast = parse_macro_input!(input as DeriveInput);

    let maybe_output = attempt_derive_list(ast);
    let output = 
        match maybe_output {
            Ok(output) => output,
            Err(errors) => {
                let compile_errors = errors.into_iter().map(|e| e.to_compile_error());
                quote!(#(#compile_errors)*)
            }
        };

    output.into()
}
fn attempt_derive_list(ast: DeriveInput) -> Result<TokenStream, Vec<syn_v2::Error>> {
    let mut all_errors = vec![];

    let struct_data =
        match ast.data {
            Data::Struct(struct_date) => struct_date,
            _ => {
                all_errors.push(syn_v2::Error::new(ast.ident.span(), "Invalid body expected struct"));
                return Err(all_errors)
            }
        };

    let mut builder = ListTraitBuilder::new(ast.ident);
    


    for field in struct_data.fields {
        let field_attr = 
            match AttributeAttribute::from_attrs(field.ident.span(), field.attrs.clone()) {
                Ok(attr) => attr,
                Err(ref mut errors) => {
                    all_errors.append(errors);
                    continue;
                }
            };

        builder.check_field(field, field_attr);
        
    }

    let output = builder.build();

    match all_errors.len() {
        0 => Ok(output),
        _ => Err(all_errors)
    }
}









struct BuilderParts {
    builder_name: Ident,
    field_declaration: TokenStream,
    field_expansion: TokenStream,
    concat_parts: TokenStream
}
impl BuilderParts {
    fn new(struct_name: &Ident) -> Self {
        Self {
            builder_name: format_ident!("{struct_name}Builder"),
            field_declaration: TokenStream::new(),
            field_expansion: TokenStream::new(),
            concat_parts: TokenStream::new()
        }
    }
    fn generate_builder(self) -> (TokenStream, Ident) {
        let Self { builder_name, field_declaration, field_expansion, concat_parts } = self;

        let declaration = 
            quote!{
                struct #builder_name<V: SynVersion> {
                    #field_declaration
                }
                impl<V: SynVersion> #builder_name<V> {
                    fn new(location: Span) -> Self {
                        Self {
                            #field_expansion
                        }
                    } 
                }
                impl<V: SynVersion> Concat for #builder_name<V> {
                    const NO_DUPLICATES: bool = false;

                    fn concat(&mut self, other: Self) {
                        #concat_parts
                    }
                }
            };

        (declaration, builder_name)
    }
}


struct Validation {
    validate_arguments: TokenStream,
    expansion: TokenStream
}
impl Validation {
    fn new() -> Self {
        Self {
            validate_arguments: TokenStream::new(),
            expansion: TokenStream::new()
        }
    }
}

struct TryFrom {
    match_branches: TokenStream
}
impl TryFrom {
    fn new() -> Self {
        Self {
            match_branches: TokenStream::new()
        }
    }
}

struct MacroBase {
    struct_name: Ident,
    builder_parts: BuilderParts,
    try_from: TryFrom,
    validation: Validation,
}
impl MacroBase {
    fn new(struct_name: Ident) -> Self {
        Self {
            struct_name: struct_name.clone(),
            builder_parts: BuilderParts::new(&struct_name),
            try_from: TryFrom::new(),
            validation: Validation::new()
        }
    }

    fn check_field(&mut self, field: Field, attribute: AttributeAttribute) {
        let Self { builder_parts, try_from, validation, ..} = self;

        let field_name = field.ident.unwrap();
        let field_type = field.ty;
        
        {
            let field_decl = quote!{ #field_name: ArgResult<<#field_type as TryFromMeta<V>>::InitialType>, };
            builder_parts.field_declaration.append_all(field_decl);
        }

        {
            let field_expansion = quote!{ #field_name: ArgResult::new(location), };
            builder_parts.field_expansion.append_all(field_expansion);
        }
        
        {
            let concat_part = quote!{self.#field_name.concat(other.#field_name);};
            builder_parts.concat_parts.append_all(concat_part);
        }

        let field_name_str = 
            match attribute.name {
                Some(name) => name,
                None => field_name.to_string()
            };
        {
            let branch = 
                quote!{
                    #field_name_str => {
                        let value = <#field_type as TryFromMeta<V>>::try_from_meta(arg);
                        builder.#field_name.concat(value);
                    }
                };
            try_from.match_branches.append_all(branch);
        }
   
        let field_type_str = field_name.to_string();
        {
            let normal_validation = 
                quote!{
                    let mut #field_name = <#field_type as TryFromMeta<V>>::validate(builder.#field_name, #field_type_str);
                    if let Err(ref mut errors) = #field_name {
                        state.errors.append(errors);
                    }
                };

            let validate_field = 
                match attribute.default {
                    Some(arg) => {
                        let x = 
                            match arg.0 {
                                Default::UseSelfDefault => quote!{ <#field_type as Default>::default() },
                                Default::ChooseDefault(path) => quote![ #path() ]
                            };

                        quote!{
                            let mut #field_name = 
                            match builder.#field_name.value.is_none() && builder.#field_name.found_with_errors() == false {
                                true => Ok(#x),
                                false => {
                                    #normal_validation

                                    #field_name
                                }
                            };
                        }
                    }
                    None => normal_validation
                };
            validation.validate_arguments.append_all(validate_field);
        }
        
        {
            let field_error = format!("failed to deserialize '{field_name_str}'");
            let field_expansion = quote!{ #field_name: #field_name.expect(#field_error), };
            validation.expansion.append_all(field_expansion);
        }
       
    }
}

struct AttributeTraitBuilder {
    container_attr: AttributeAttribute,
    base: MacroBase
}
impl AttributeTraitBuilder {
    fn new(struct_name: Ident, container_attr: AttributeAttribute) -> Self {
        Self {
            container_attr,
            base: MacroBase::new(struct_name)
        }
    }
    fn check_field(&mut self, field: Field, attribute: AttributeAttribute) {
        self.base.check_field(field, attribute);
    }

    fn build(self) -> TokenStream {
        let Self {
            container_attr, 
            base:
                MacroBase { 
                    struct_name, 
                    builder_parts, 
                    try_from, 
                    validation
                } 
            } = self;

        let set_default = 
            match container_attr.default {
                Some(CustomArg(Default::UseSelfDefault)) => quote!{ return Ok(<Self as Default>::default()) },
                Some(CustomArg(Default::ChooseDefault(path))) => quote!{ return Ok(#path()) },
                None => quote!()
            };

        let (builder_decl, builder_name) = builder_parts.generate_builder();
        

        let mut all_attribute_impls = TokenStream::new();
        #[cfg(feature = "syn_1")]
        {
            let attr_impl =
                quote!{
                    use derive_attribute::Syn1;
                    impl Attribute<Syn1> for #struct_name {} 
                };
            all_attribute_impls.append_all(attr_impl);
        }
        #[cfg(feature = "syn_2")]
        {
            let attr_impl =
                quote!{
                    use derive_attribute::Syn2;
                    impl Attribute<Syn2> for #struct_name {} 
                };
            all_attribute_impls.append_all(attr_impl);
        }


        let name = 
            match container_attr.name {
                Some(name) => name,
                None => struct_name.to_string().to_case(Case::Snake)
            };

        let try_from_fn = generate_try_from_meta(format_ident!("deserialize_attr_args"), &builder_name, try_from);
        let validation_fn = generate_validate(validation, set_default, format_ident!("MissingAttribute"));

        quote!{
            const _: () = {
                use derive_attribute::{AttributeName, TryFromMeta, Attribute, GetSpan, Concat, Error, ErrorMsg::*, SynVersion, ArgResult, reexports::proc_macro2::Span};

                impl AttributeName for #struct_name {
                    const NAME: &'static str = #name;
                }

                #all_attribute_impls

                #builder_decl

                impl<V: SynVersion> TryFromMeta<V> for #struct_name {
                    type InitialType = #builder_name<V>;
                
                    type Metadata = V::Attribute;
                    
                    #try_from_fn
                
                    #validation_fn
                }
                
            };
        }
     } 

}


struct ListTraitBuilder {
    base: MacroBase
}
impl ListTraitBuilder {
    fn new(struct_name: Ident) -> Self {
        Self {
            base: MacroBase::new(struct_name)
        }
    }
    fn check_field(&mut self, field: Field, attribute: AttributeAttribute) {
        self.base.check_field(field, attribute);
    }
    fn build(self) -> TokenStream {
        let Self {
            base: 
                MacroBase { 
                    struct_name, 
                    builder_parts, 
                    try_from, 
                    validation
                } 
            } = self;

        let (builder_decl, builder_name) = builder_parts.generate_builder();
        
        let try_from_fn = generate_try_from_meta(format_ident!("deserialize_list_args"), &builder_name, try_from);
        let validation_fn = generate_validate(validation, quote!(), format_ident!("MissingArg"));

        quote!{
            const _: () = {
                use derive_attribute::{AttributeName, TryFromMeta, Attribute, GetSpan, Concat, Error, ErrorMsg::*, SynVersion, ArgResult, reexports::proc_macro2::Span};


                #builder_decl

                impl<V: SynVersion> TryFromMeta<V> for #struct_name {
                    type InitialType = #builder_name<V>;
                
                    type Metadata = V::ArgMeta;
                    
                    #try_from_fn
                
                    #validation_fn
                }
                
            };
        }
        
    }

}



fn generate_try_from_meta(deserialize_args: Ident, builder_name: &Ident, try_from: TryFrom) -> TokenStream {
    let TryFrom { match_branches } = try_from;
    quote!{
        fn try_from_meta(arg_meta: Self::Metadata) -> ArgResult<Self::InitialType> {
            let mut result = ArgResult::new(arg_meta.get_span());
    
            let mut builder = #builder_name::new(arg_meta.get_span());
    
            let attribute_args = 
                match V::#deserialize_args(&arg_meta) {
                    Some(args) => args,
                    None => {
                        result.add_error(InvalidType { expected: "list" });
                        return result
                    }
                };
            
    
            for arg in attribute_args {
                let key = V::deserialize_key(&arg).expect("key failed");
                match key.as_str() {
                    #match_branches

                    _ => result.errors.push(Error::new(arg.get_span(), InvalidArg))
                }
            }
            result.add_value(builder);
            result
        }
    }
}

fn generate_validate(validate: Validation, set_default: TokenStream, error_type: Ident) -> TokenStream {
    let Validation { validate_arguments, expansion } = validate;
    quote!{
        fn validate(state: ArgResult<Self::InitialType>, arg_name: &'static str) -> Result<Self, Vec<Error>> {
            let mut state = state;

            if state.value.is_none() && state.found_with_errors() == false {
                #set_default
            }
    
            // let mut builder = 
            //     match state.value {
            //         Some(value) => value,
            //         None => return Err(vec![Error::new(state.location, #error_type(arg_name))])
            //     };

            let mut builder =
                match state.found_with_errors() {
                    true => return Err(state.errors),
                    false if state.value.is_none() => {
                        state.add_error(#error_type(arg_name));
                        return Err(state.errors);
                    }
                    false => state.value.unwrap()
                };

            #validate_arguments
    
    
            match state.errors.len() {
                0 => Ok(Self { #expansion }),
                _ => Err(state.errors)
            }
        }
    }
}



#[derive(Debug, Default)]
struct AttributeAttribute {
    name: Option<String>,
    default: Option<CustomArg<Default>>,
}

struct AttributeAttributeBuilder<V: SynVersion> {
    name: ArgResult<<Option<String> as TryFromMeta<V>>::InitialType>,
    default: ArgResult<<Option<CustomArg<Default>> as TryFromMeta<V>>::InitialType>,
}
impl<V: SynVersion> AttributeAttributeBuilder<V> {
    fn new(location: Span) -> Self {
        Self { 
            name: ArgResult::new(location),
            default: ArgResult::new(location),
        }
    }
}
impl<V: SynVersion> Concat for AttributeAttributeBuilder<V> {
    const NO_DUPLICATES: bool = false;
    fn concat(&mut self, other: Self) {
        self.name.concat(other.name);
        self.default.concat(other.default);
    }
}


impl Attribute<Syn2> for AttributeAttribute {}
impl AttributeName for AttributeAttribute {
    const NAME: &'static str = "attr";
}
impl<V: SynVersion> TryFromMeta<V> for AttributeAttribute {
    type InitialType = AttributeAttributeBuilder<V>;

    type Metadata = V::Attribute;
    fn try_from_meta(arg_meta: Self::Metadata) -> ArgResult<Self::InitialType> {
        let mut result = ArgResult::new(arg_meta.get_span());

        let mut builder = AttributeAttributeBuilder::new(arg_meta.get_span());

        let attribute_args = 
            match V::deserialize_attr_args(&arg_meta) {
                Some(args) => args,
                None => {
                    result.add_error(InvalidType { expected: "list" });
                    return result
                }
            };


        for arg in attribute_args {
            let key = V::deserialize_key(&arg).expect("key failed");

            match key.as_str() {
                "name" => {
                    let value = <Option<String> as TryFromMeta<V>>::try_from_meta(arg);
                    builder.name.concat(value);
                }
                "default" => {
                    let value = <Option<CustomArg<Default>> as TryFromMeta<V>>::try_from_meta(arg);
                    builder.default.concat(value);
                }

                _ => result.errors.push(Error::new(arg.get_span(), InvalidArg))
            };

        }
        result.add_value(builder);
        result
    }

    fn validate(state: ArgResult<Self::InitialType>, arg_name: &'static str) -> Result<Self, Vec<Error>> {
        let mut state = state;

        if state.value.is_none() && state.found_with_errors() == false {
            return Ok(Self::default())
        }

        let builder = 
            match state.value {
                Some(value) => value,
                None => return Err(vec![Error::new(state.location, MissingAttribute(arg_name))])
            };

        let mut maybe_name = <Option<String> as TryFromMeta<V>>::validate(builder.name, "name");
        if let Err(ref mut errors) = maybe_name {
            state.errors.append(errors);
        }

        let mut maybe_default = <Option<CustomArg<Default>> as TryFromMeta<V>>::validate(builder.default, "default");
        if let Err(ref mut errors) = maybe_default {
            state.errors.append(errors);
        }

        match state.errors.len() {
            0 => Ok(Self { name: maybe_name.expect("name failed"), default: maybe_default.expect("default failed") }),
            _ => Err(state.errors)
        }
    }
}

#[derive(Debug)]
enum Default {
    UseSelfDefault,
    ChooseDefault(Ident)
}
impl<V: SynVersion> CustomArgFromMeta<V> for Default {
    fn try_from_meta(meta: V::ArgMeta) -> Result<Self, ErrorMsg> {        
        let maybe_bool = V::deserialize_bool(&meta);
        let maybe_path = V::deserialize_string(&meta);
        
        match (maybe_bool, maybe_path) {
            (Some(is_default), _) if is_default => Ok(Self::UseSelfDefault),
            (_, Some(path)) => Ok(Self::ChooseDefault(format_ident!("{path}"))),
            _ => Err(InvalidType { expected: "boolean or path string" })
        }
    }
}