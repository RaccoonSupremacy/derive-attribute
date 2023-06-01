use derive_attribute::{Attribute, SynVersion, ErrorMsg, TryFromMeta, required_validation, Concat, ArgResult, GetSpan};
use syn::{parse_quote, DeriveInput, spanned::Spanned};

#[allow(dead_code)]
#[derive(Debug, Default, Attribute)]
#[attr(name = "my_attr", default)]
struct MyAttr {
    #[attr(name = "first_name")]
    name: Option<String>,
    age: i32,

    feeling: Feeling
}

#[derive(Debug)]
pub enum Feeling {
    Happy,
    Neutral,
    Sad
}
impl Default for Feeling {
    fn default() -> Self { Self::Neutral }
}

impl Concat for Feeling {}
impl<V: SynVersion> TryFromMeta<V> for Feeling {
    type InitialType = Self;
    type Metadata = V::ArgMeta;
    fn try_from_meta(meta: Self::Metadata) -> ArgResult<Self::InitialType> {
        let mut arg_result = ArgResult::new(meta.get_span());

        let string = V::deserialize_string(&meta);
        
        let maybe_feeling = 
            match string {
                Some(string) => {
                    match string.to_lowercase().as_str() {
                        "happy" => Some(Self::Happy),
                        "neutral" => Some(Self::Neutral),
                        "sad" => Some(Self::Sad),
                        _ => None
                    }
                },
                None => None
            };

        let result = 
            match maybe_feeling {
                Some(feeling) => Ok(feeling),
                None => Err(ErrorMsg::InvalidType { expected: r#" "happy", "neutral", or "sad" "# })
            };

        arg_result.add_result(result);

        arg_result
    }
    fn validate(state: ArgResult<Self::InitialType>, arg_name: &'static str) -> Result<Self, Vec<derive_attribute::Error>> {
        required_validation(state, arg_name)
    }
} 




fn main() {
    let tokens: DeriveInput =
        parse_quote!{
            // #[derive(MACRO_NAME)]
            #[my_attr(first_name = "Jake", age = 24, feeling = "happy")]
            struct Test;
        };

    let my_attr = MyAttr::from_attrs(tokens.span(), tokens.attrs).unwrap();
    println!("{:?}", my_attr);
}




