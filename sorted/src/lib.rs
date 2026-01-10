#![allow(unused_imports)]

use proc_macro::TokenStream;
use proc_macro2::Span;
use syn::{spanned::Spanned, ItemEnum};

#[proc_macro_attribute]
pub fn sorted(args: TokenStream, input: TokenStream) -> TokenStream {
    let _ = args;
    let raw = input.clone();
    let raw = syn::parse_macro_input!(raw as syn::Item);
    eprintln!("raw args:{:?}", args);
    eprintln!("raw input:{:?}", raw);

    //enum or match only
    match raw {
        syn::Item::Enum(ref input_enum) => {
            if let Err(e) = check_enum_order(input_enum) {
                return e.to_compile_error().into();
            }
        }
        _ => {
            return syn::Error::new(Span::call_site(), "expected enum or match expression")
                .into_compile_error()
                .into();
        }
    }

    input
}

fn check_enum_order(input: &ItemEnum) -> Result<(), syn::Error> {
    let variants = &input.variants;
    let var_vec: Vec<_> = variants.iter().collect();
    for (i, src) in var_vec.iter().enumerate() {
        for des in &var_vec[..i] {
            if src.ident.to_string().to_lowercase() < des.ident.to_string().to_lowercase() {
                return Err(syn::Error::new(
                    src.span(),
                    &format!("{} should sort before {}", src.ident, des.ident),
                ));
            }
        }
    }

    Ok(())
}
