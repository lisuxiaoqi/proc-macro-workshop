use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, punctuated, Data, DeriveInput, Field, Fields, Token};

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let builder_name = syn::Ident::new(&format!("{}Builder", name), name.span());
    let fields: &punctuated::Punctuated<Field, Token![,]> = match input.data {
        Data::Struct(ref ds) => match ds.fields {
            Fields::Named(ref fields) => &fields.named,
            _ => unimplemented!("Only FieldsNamed are supported"),
        },
        _ => unimplemented!("Only Sturcts are supported"),
    };

    let fields_name: Vec<_> = fields
        .iter()
        .map(|f| {
            let f_name = f.ident.as_ref().unwrap();
            quote! {
                #f_name
            }
        })
        .collect();

    let fields_ty: Vec<_> = fields
        .iter()
        .map(|f| {
            let f_ty = &f.ty;
            quote! {
                #f_ty
            }
        })
        .collect();

    quote! {
        pub struct #builder_name{
            #(#fields_name:Option<#fields_ty>,)*
        }

        impl #name{
            pub fn builder()->#builder_name{
                #builder_name{
                    #(#fields_name: None,)*
                }
            }
        }

        impl #builder_name{
            #(fn #fields_name(&mut self, #fields_name: #fields_ty)->&mut Self{
                self.#fields_name = Some(#fields_name);
                self
            })*
        }
    }
    .into()
}
