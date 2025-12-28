use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse_macro_input, punctuated, AngleBracketedGenericArguments, Data, DeriveInput, Field,
    Fields, GenericArgument, LitStr, Path, PathArguments, Token, Type, TypePath,
};

#[proc_macro_derive(Builder, attributes(builder))]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let mut fields_must = Vec::new();
    let mut fields_must_ty = Vec::new();
    let mut fields_opt = Vec::new();
    let mut fields_opt_ty = Vec::new();
    let mut fields_setter = Vec::new();
    let mut fields_init = Vec::new();

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
                        //fields opt
                        fields_opt_ty.push(quote! {#f_opt_ty});

                        fields_init.push(quote! {
                            #f_name : None
                        });

                        //fields setter
                        fields_setter.push(quote! {
                            fn #f_name(&mut self, #f_name : #f_opt_ty)->&mut Self{
                                self.#f_name = Some(Some(#f_name));
                                self
                            }
                        });
                    }
                }
            } else {
                fields_must.push(quote! {#f_name});
                let f_must_ty = &f.ty;
                fields_must_ty.push(quote! {#f_must_ty});

                let mut is_each = false;

                //parse attribute
                for attr in &f.attrs {
                    if attr.path().is_ident("builder") {
                        if let Err(e)  = attr.parse_nested_meta(|meta| {
                            if meta.path.is_ident("each") {
                                let each_type = parse_vec_type(&f.ty).unwrap();
                                let value = meta.value()?;
                                let s: LitStr = value.parse()?;
                                let setter_each_name = syn::Ident::new(&s.value(), s.span());

                                is_each = true;
                                fields_setter.push(quote! {
                                    fn #setter_each_name(&mut self, #setter_each_name:#each_type)->&mut Self{
                                        self.#f_name.get_or_insert_with(Vec::new).push(#setter_each_name);
                                        self
                                    }
                                });

                                fields_init.push(quote!{
                                    #f_name : Some(Vec::<#each_type>::new())
                                });
                                Ok(())
                            } else {
                                Err(syn::Error::new_spanned(&attr.meta,r#"expected `builder(each = "...")`"#))
                            }
                        }){
                            return e.into_compile_error().into();
                        }
                    }
                }

                //fields setter
                if !is_each {
                    fields_setter.push(quote! {
                        fn #f_name(&mut self, #f_name : #f_must_ty)->&mut Self{
                            self.#f_name = Some(#f_name);
                            self
                        }
                    });

                    fields_init.push(quote! {
                        #f_name : None
                    });
                }
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
                    #(#fields_init,)*
                }
            }
        }

        impl #builder_name{
            #(#fields_setter)*

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

fn parse_vec_type(vec_type: &syn::Type) -> Option<&Type> {
    let Type::Path(TypePath {
        path: Path { segments, .. },
        ..
    }) = vec_type
    else {
        return None;
    };

    let segment = segments.last()?;
    if segment.ident != "Vec" {
        return None;
    }

    //get inner option type
    let PathArguments::AngleBracketed(AngleBracketedGenericArguments { args, .. }) =
        &segment.arguments
    else {
        return None;
    };

    match args.first()? {
        GenericArgument::Type(f_opt_ty) => Some(f_opt_ty),
        _ => None,
    }
}
