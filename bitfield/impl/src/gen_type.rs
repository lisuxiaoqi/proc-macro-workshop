use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;

pub fn generate(_: TokenStream) -> TokenStream {
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
