use derive_attribute::{Attribute, List};
use syn::{parse_quote, DeriveInput, spanned::Spanned};

#[allow(dead_code)]
#[derive(Debug, Default, Attribute)]
#[attr(name = "my_attr", default)]
struct MyAttr {
    #[attr(name = "first_name")]
    name: Option<String>,
    age: i32,
    hobbies: Vec<String>,
    list: Option<MyList>,
}
    #[allow(dead_code)]
    #[derive(Default, Debug, List)]
    struct MyList {
        a: u8
    }

fn main() {
    let tokens: DeriveInput =
        parse_quote!{
            // #[derive(MACRO_NAME)]
            #[my_attr(first_name = "Jake", age = 1, hobbies = ["running", "baseball"])]
            #[my_attr(list(a = 1))]
            struct Test;
        };

    let my_attr = MyAttr::from_attrs(tokens.span(), tokens.attrs).unwrap();
    println!("{:?}", my_attr);
}




