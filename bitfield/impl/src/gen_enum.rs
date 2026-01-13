use crate::gen_type;
use proc_macro::TokenStream;
use quote::quote;
use syn::ItemEnum;

pub fn generate(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as ItemEnum);
    let name = &input.ident;
    let face: syn::Type = syn::parse_quote!(#name);
    //eprintln!("ItemEnum:{:?}", &input);
    //get bits, base
    let bits = get_enum_bits(&input);
    let base = gen_type::get_base(bits);
    let specifier = gen_type::gen_specifier(&name, &base, &face, bits);

    let (facearm, basearm) = gen_trans_arms(&input);
    let transfer = gen_trans(&base, &face, &facearm, &basearm);

    //gen specifier
    quote! {
        #transfer
        #specifier
    }
    .into()
}

fn gen_trans_arms(item: &syn::ItemEnum) -> (proc_macro2::TokenStream, proc_macro2::TokenStream) {
    let mut facearm = proc_macro2::TokenStream::new();
    let mut basearm = proc_macro2::TokenStream::new();
    let name = &item.ident;
    for var in &item.variants {
        if let Some((_, syn::Expr::Lit(litexpr))) = &var.discriminant {
            if let syn::Lit::Int(intlit) = &litexpr.lit {
                let val_usize = intlit.base10_parse::<usize>().unwrap();
                let val = syn::LitInt::new(&val_usize.to_string(), intlit.span());
                let key = &var.ident;

                facearm.extend(quote! {
                    #name::#key => #val,
                });

                basearm.extend(quote! {
                    #val => #name::#key,
                });
            }
        }
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

fn get_enum_bits(item: &syn::ItemEnum) -> usize {
    let v1 = item.variants.last().unwrap();
    if let Some((_, syn::Expr::Lit(lit))) = &v1.discriminant {
        if let syn::Lit::Int(int_lit) = &lit.lit {
            let token = int_lit.token().to_string();
            let token = token.replace("0b", "");
            return token.len();
        }
    }

    panic!("Enum Field must be ExprLit")
}
