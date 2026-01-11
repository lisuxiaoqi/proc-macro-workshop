use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{quote, ToTokens};
use syn::spanned::Spanned;

#[proc_macro_attribute]
pub fn bitfield(args: TokenStream, input: TokenStream) -> TokenStream {
    let _ = args;
    let _ = input;

    let input = syn::parse_macro_input!(input as syn::ItemStruct);
    let name = &input.ident;

    let syn::Fields::Named(fields) = &input.fields else {
        return syn::Error::new(input.span(), "Only FieldsNames are supported")
            .to_compile_error()
            .into();
    };

    let bit_size = calsize(fields);
    let bytes = quote! {(#bit_size)/8};
    quote! {
        #[repr(C)]
        pub struct #name{
            data:[u8;#bytes],
        }
    }
    .into()
}

fn calsize(fields: &syn::FieldsNamed) -> proc_macro2::TokenStream {
    fields.named.iter().fold(quote! {0usize}, |mut acc, f| {
        let mut bits = quote! {};
        if let syn::Type::Path(path_type) = &f.ty {
            if let Some(t) = path_type.path.segments.last() {
                let enum_name = &t.ident;
                bits = quote! {
                    +#enum_name::BITS
                };
            } else {
                eprintln!(
                    "TypePath segments has no last:{}",
                    path_type.to_token_stream()
                );
            }
        }
        acc.extend(bits);
        acc
    })
}

#[proc_macro]
pub fn expand_types(_: TokenStream) -> TokenStream {
    let mut output = Vec::new();
    for i in 1..=64usize {
        let name = syn::Ident::new(&format!("B{}", i), Span::call_site());
        let bi = quote! {
            pub enum #name{}
            impl Specifier for #name{
                const BITS:usize = #i;
            }
        };
        output.push(bi);
    }

    quote! {
        #(#output)*
    }
    .into()
}
