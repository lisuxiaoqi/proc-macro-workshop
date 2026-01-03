use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse_macro_input, spanned::Spanned, Data, DeriveInput, Expr, ExprLit, Field, Fields,
    GenericParam, Generics, Lit, Meta, Type,
};

#[proc_macro_derive(CustomDebug, attributes(debug))]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    //ident
    let name = &input.ident;

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
    for field in fields {
        let (ty_ident, ok) = is_generic_type(field, gs);
        if ok {
            trait_bounds.push(quote! {#ty_ident : std::fmt::Debug});
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

fn is_generic_type(f: &Field, gs: &Generics) -> (syn::Ident, bool) {
    let mut ty_ident = syn::Ident::new("tmp", f.span());
    let mut is_generic = false;
    if let Type::Path(tp) = &f.ty {
        ty_ident = tp.path.segments.last().unwrap().ident.clone();
        //eprintln!("tp.path.segments.last:{}", tp.to_token_stream());
        for p in &gs.params {
            if let GenericParam::Type(tp) = p {
                //eprintln!("type param:{}", tp.to_token_stream());
                if tp.ident == ty_ident {
                    is_generic = true;
                }
            }
        }
    }
    (ty_ident, is_generic)
}
