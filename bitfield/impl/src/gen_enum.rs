use crate::gen_type;
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::ItemEnum;

pub fn generate(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as ItemEnum);
    let name = &input.ident;
    let face: syn::Type = syn::parse_quote!(#name);
    //eprintln!("ItemEnum:{:?}", &input);
    //get bits, base
    let bits;
    match get_enum_bits(&input) {
        Ok(v) => bits = v,
        Err(e) => return e.to_compile_error().into(),
    }
    let base = gen_type::get_base(bits);
    let specifier = gen_type::gen_specifier(&name, &base, &face, bits);

    let (facearm, basearm) = gen_trans_arms(&input);
    let transfer = gen_trans(&base, &face, &facearm, &basearm);
    let check = gen_check(&input);

    //gen specifier
    quote! {
        #transfer
        #specifier
        #check
    }
    .into()
}

fn gen_check(item: &syn::ItemEnum) -> proc_macro2::TokenStream {
    let name = &item.ident;
    let mut flag = quote! {};
    for var in &item.variants {
        let key = &var.ident;
        flag.extend(quote! {#name::#key as usize == 0 ||});
    }
    flag.extend(quote! {false});
    quote! {
        impl bitfield::checks::CheckRange<bitfield::checks::RangeWrapper<{#flag}>> for #name{}
    }
}
fn gen_trans_arms(item: &syn::ItemEnum) -> (proc_macro2::TokenStream, proc_macro2::TokenStream) {
    let mut facearm = proc_macro2::TokenStream::new();
    let mut basearm = proc_macro2::TokenStream::new();
    let name = &item.ident;
    for var in &item.variants {
        let key = &var.ident;
        let lower_key: syn::Ident = syn::Ident::new(&key.to_string().to_lowercase(), key.span());
        let val = quote! {
            #name::#key as <#name as Specifier>::Base
        };

        facearm.extend(quote! {
            #name::#key => #val,
        });

        basearm.extend(quote! {
            #lower_key if #lower_key == #val => #name::#key,
        });
    }
    (facearm, basearm)
}

fn gen_trans(
    base: &syn::Type,
    face: &syn::Type,
    facearm: &proc_macro2::TokenStream,
    basearm: &proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    quote! {
        impl FromTrans<#face> for #base{
            fn fromt(v: #face)->#base{
                match v{
                    #facearm
                    _=>unreachable!("unreacheable value"),
                }
            }
        }

        impl FromTrans<#base> for #face {
            fn fromt(v:#base) -> #face {
                match v{
                    #basearm
                    _=>unreachable!("unreacheable value"),
                }
            }
        }
    }
}

fn get_enum_bits(item: &syn::ItemEnum) -> Result<usize, syn::Error> {
    let mut len = item.variants.len();
    if len % 2 != 0 {
        return Err(syn::Error::new(
            Span::call_site(),
            "BitfieldSpecifier expected a number of variants which is a power of 2",
        ));
    }

    let mut i = 0;
    while len != 0 {
        len = len >> 1;
        i += 1;
    }

    Ok(i - 1)
}
