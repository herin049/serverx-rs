use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse2, Expr, LitStr};

fn is_valid_namespace(namespace: &str) -> bool {
    namespace.len() > 0
        && namespace.chars().all(|c| {
            c == '_' || c == '-' || (c >= 'a' && c <= 'z') || (c >= '0' && c <= '9') || c == '.'
        })
}

fn is_valid_path(path: &str) -> bool {
    path.len() > 0
        && path.chars().all(|c| {
            c == '_'
                || c == '-'
                || (c >= 'a' && c <= 'z')
                || (c >= '0' && c <= '9')
                || c == '/'
                || c == '.'
        })
}

pub fn identifier_macro_impl(input: TokenStream) -> TokenStream {
    let input_str: LitStr = parse2(input).expect("unable to parse macro input");
    let input_value = input_str.value();
    let str_value = input_value.as_str();
    if let Some(delim_pos) = str_value.chars().position(|c| c == ':') {
        if !is_valid_namespace(&str_value[..delim_pos]) {
            panic!("{} has an invalid namespace", str_value);
        } else if !is_valid_path(&str_value[(delim_pos + 1)..]) {
            panic!("{} has an invalid path", str_value);
        }
        quote! {
            identifier::Identifier::new(#input_str.to_string(), #delim_pos)
        }
    } else if !is_valid_path(str_value) {
        panic!("{} has an invalid path", str_value);
    } else {
        let mut str = "minecraft:".to_string();
        let delim_pos = str.len() - 1;
        str.push_str(str_value);
        quote! {
            identifier::Identifier::new(#str.to_string(), #delim_pos)
        }
    }
}
