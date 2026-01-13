use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;

pub fn generate(_: TokenStream) -> TokenStream {
    let mut output = Vec::new();
    for i in 1..=64usize {
        let name = syn::Ident::new(&format!("B{}", i), Span::call_site());
        let base = get_base(i);
        let bi = gen_type(&name, &base, i);
        output.push(bi);
    }

    quote! {
        #(#output)*
    }
    .into()
}

fn get_base(i: usize) -> syn::Type {
    match i {
        1..=8 => syn::parse_quote! {u8},
        9..=16 => syn::parse_quote! {u16},
        17..=32 => syn::parse_quote! {u32},
        33..=64 => syn::parse_quote! {u64},
        _ => unreachable!("invalide base range"),
    }
}

pub fn gen_type(name: &syn::Ident, base: &syn::Type, bits: usize) -> proc_macro2::TokenStream {
    quote! {
        pub enum #name{}
        impl Specifier for #name{
            const BITS:usize = #bits;
            type Base = #base;
        }
    }
}
