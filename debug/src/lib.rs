use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Expr, ExprLit, Field, Fields, Lit, Meta};

#[proc_macro_derive(CustomDebug, attributes(debug))]
pub fn derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = &input.ident;
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

    quote! {
        impl std::fmt::Debug for #name{
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
