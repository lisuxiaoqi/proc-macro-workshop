use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{
    parse_macro_input, punctuated::Punctuated, Data, DeriveInput, Expr, ExprLit, Field, Fields,
    GenericArgument, GenericParam, Lit, Meta, PathArguments, Token, Type, TypeParam, TypePath,
};

#[proc_macro_derive(CustomDebug, attributes(debug))]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    //ident
    let name = &input.ident;
    eprintln!("--Start parse input:{}", name);

    //generics
    let gs = &input.generics;
    let (g_impl, g_ty, _g_where) = gs.split_for_impl();
    let mut trait_bounds = Vec::new();

    //fields
    let fields = match &input.data {
        Data::Struct(ds) => match &ds.fields {
            Fields::Named(fs) => &fs.named,
            _ => unimplemented!("Only Named Fields are supported"),
        },
        _ => unimplemented!("Only DataStruct are supported!"),
    };

    let _f_names: Vec<_> = fields
        .iter()
        .map(|f| {
            let f_name = f.ident.as_ref().unwrap();
            quote! {
                #f_name
            }
        })
        .collect();

    let f_formats: Vec<_> = fields
        .iter()
        .map(|f| {
            let f_name = f.ident.as_ref().unwrap();
            let f_arg = parse_arg(f);

            quote! {
                .field(stringify!(#f_name), &format_args!(#f_arg, &self.#f_name))
            }
        })
        .collect();

    //trait bounds
    for gp in &gs.params {
        if let GenericParam::Type(tp) = gp {
            eprintln!("GenericParam:{}", tp.to_token_stream());
            let ty_ident = &tp.ident;
            if !is_in_phantom(tp, fields) {
                trait_bounds.push(quote! {#ty_ident : std::fmt::Debug});
            }
        }
    }

    let mut where_clause = quote! {};
    if !trait_bounds.is_empty() {
        where_clause = quote! {where #(#trait_bounds),*};
    }

    quote! {
        impl #g_impl std::fmt::Debug for #name #g_ty #where_clause{
            fn fmt(&self, f:&mut std::fmt::Formatter<'_>) -> std::fmt::Result{
                f.debug_struct(stringify!(#name))
                #(#f_formats)*
                .finish()
            }
        }
    }
    .into()
}

fn parse_arg(f: &Field) -> String {
    let args = "{:?}";
    for attr in &f.attrs {
        if let Meta::NameValue(nv) = &attr.meta {
            if nv.path.is_ident("debug") {
                if let Expr::Lit(ExprLit { lit, .. }) = &nv.value {
                    if let Lit::Str(lit_s) = &lit {
                        return lit_s.value();
                    }
                }
            }
        }
    }
    args.into()
}

fn is_in_phantom(tp: &TypeParam, fields: &Punctuated<Field, Token![,]>) -> bool {
    for f in fields {
        eprintln!("Parse field:{}", &f.ident.as_ref().unwrap());
        if !is_phantom(tp, &f.ty) {
            eprintln!("field type is not phantomData:{}", &f.ty.to_token_stream());
            return false;
        }
    }
    true
}

fn is_phantom(tp: &TypeParam, ty: &Type) -> bool {
    if let Type::Path(type_path) = ty {
        let segs = &type_path.path.segments;
        if let Some(last_seg) = segs.last() {
            eprintln!("last_seg:{}", &last_seg.ident);
            //PhantomData<T>
            if last_seg.ident == "PhantomData" {
                if let PathArguments::AngleBracketed(args) = &last_seg.arguments {
                    for arg in &args.args {
                        if let GenericArgument::Type(Type::Path(TypePath { path, .. })) = arg {
                            if path.is_ident(&tp.ident) {
                                return true;
                            }
                        }
                    }
                }
            }

            //recursive args
            match &last_seg.arguments {
                PathArguments::AngleBracketed(args) => {
                    eprintln!("\tAngleBracketed args:{}", args.to_token_stream());
                    for arg in &args.args {
                        if let GenericArgument::Type(t) = arg {
                            return is_phantom(tp, t);
                        }
                    }
                }
                PathArguments::None => {
                    if let Type::Path(TypePath { path, .. }) = ty {
                        if path.is_ident(&tp.ident) {
                            return false;
                        }
                    }
                }
                _ => (),
            }
        }
    }
    true
}
