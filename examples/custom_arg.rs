use derive_attribute::{Attribute, CustomArgFromMeta, SynVersion, ErrorMsg, CustomArg};
use syn::{parse_quote, DeriveInput, spanned::Spanned};


#[allow(dead_code)]
#[derive(Debug, Default, Attribute)]
#[attr(name = "my_attr", default)]
struct MyAttr {
    #[attr(name = "first_name")]
    name: Option<String>,
    age: i32,

    feeling: CustomArg<Feeling>
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

impl<V: SynVersion> CustomArgFromMeta<V> for Feeling {
    fn try_from_meta(meta: V::ArgMeta) -> Result<Self, derive_attribute::ErrorMsg> {
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

        match maybe_feeling {
            Some(feeling) => Ok(feeling),
            None => Err(ErrorMsg::InvalidType { expected: r#" "happy", "neutral", or "sad" "# })
        }
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




