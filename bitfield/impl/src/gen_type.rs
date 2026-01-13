use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;

//generate b1~b64, bool
pub fn generate(_: TokenStream) -> TokenStream {
    let mut output = Vec::new();
    for i in 1..=64usize {
        let name = syn::Ident::new(&format!("B{}", i), Span::call_site());
        //generate type
        output.push(quote! {pub enum #name{}});

        //generate specifier
        let base = get_base(i);
        let bi = gen_specifier(&name, &base, &base, i);
        output.push(bi);
    }

    //for bool
    output.push(quote! {
        impl Specifier for bool{
            const BITS:usize = 1;
            type Base = u8;
            type Face = bool;
        }

        impl FromTrans<bool> for u8{
            fn fromt(v:bool)->u8{
                match v{
                    false=>0,
                    _=>1,
                }
            }
        }

        impl FromTrans<u8> for bool{
            fn fromt(v:u8)->bool{
                match v{
                    0=>false,
                    1=>true,
                    _=>unreachable!(),
                }
            }
        }
    });

    quote! {
        #(#output)*
    }
    .into()
}

pub fn get_base(i: usize) -> syn::Type {
    match i {
        1..=8 => syn::parse_quote! {u8},
        9..=16 => syn::parse_quote! {u16},
        17..=32 => syn::parse_quote! {u32},
        33..=64 => syn::parse_quote! {u64},
        _ => unreachable!("invalide base range"),
    }
}

pub fn gen_specifier(
    name: &syn::Ident,
    base: &syn::Type,
    face: &syn::Type,
    bits: usize,
) -> proc_macro2::TokenStream {
    quote! {
        impl Specifier for #name{
            const BITS:usize = #bits;
            type Base = #base;
            type Face = #face;
        }
    }
}
