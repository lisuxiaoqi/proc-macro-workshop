use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse_macro_input, punctuated, AngleBracketedGenericArguments, Data, DeriveInput, Field,
    Fields, GenericArgument, Path, PathArguments, Token, Type, TypePath,
};

#[proc_macro_derive(Builder)]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let mut fields_must = Vec::new();
    let mut fields_must_ty = Vec::new();
    let mut fields_opt = Vec::new();
    let mut fields_opt_ty = Vec::new();

    let builder_name = syn::Ident::new(&format!("{}Builder", name), name.span());
    let fields: &punctuated::Punctuated<Field, Token![,]> = match input.data {
        Data::Struct(ref ds) => match ds.fields {
            Fields::Named(ref fields) => &fields.named,
            _ => unimplemented!("Only FieldsNamed are supported"),
        },
        _ => unimplemented!("Only Sturcts are supported"),
    };

    //all fields name and type
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

    //for fields option/must
    for f in fields {
        if let Type::Path(TypePath {
            path: Path { segments, .. },
            ..
        }) = &f.ty
        {
            let f_name = f.ident.as_ref().unwrap();
            let segment = segments.last().unwrap();
            if segment.ident == "Option" {
                fields_opt.push(quote! {#f_name});

                //get inner option type
                if let PathArguments::AngleBracketed(AngleBracketedGenericArguments {
                    args, ..
                }) = &segment.arguments
                {
                    let arg = args.first().unwrap();
                    if let GenericArgument::Type(f_opt_ty) = &arg {
                        fields_opt_ty.push(quote! {#f_opt_ty});
                    }
                }
            } else {
                fields_must.push(quote! {#f_name});
                let f_must_ty = &f.ty;
                fields_must_ty.push(quote! {#f_must_ty});
            }
        }
    }

    quote! {
        use std::boxed::Box;
        use std::error::Error;
        use std::result::Result;

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
            #(fn #fields_must(&mut self, #fields_must: #fields_must_ty)->&mut Self{
                self.#fields_must = Some(#fields_must);
                self
            })*

            #(fn #fields_opt(&mut self, #fields_opt: #fields_opt_ty)->&mut Self{
                self.#fields_opt = Some(Some(#fields_opt));
                self
            })*

            pub fn build(&mut self) -> Result<#name, Box<dyn Error>>{
                #(
                    if self.#fields_must.is_none(){
                        return Err(Box::<dyn Error>::from(format!("{} is None", stringify!(#fields_name))))
                    }
                )*

                Ok(#name{
                   #(#fields_must : self.#fields_must.take().unwrap(),)*
                   #(#fields_opt : self.#fields_opt.take().unwrap_or_else(||None),)*
                })

            }
        }
    }
    .into()
}
