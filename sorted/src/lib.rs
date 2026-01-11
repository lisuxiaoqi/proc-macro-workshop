use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{spanned::Spanned, visit_mut::VisitMut, ItemEnum};

#[proc_macro_attribute]
pub fn sorted(args: TokenStream, input: TokenStream) -> TokenStream {
    let _ = args;
    let raw = input.clone();
    let raw = syn::parse_macro_input!(raw as syn::Item);
    //eprintln!("raw args:{:?}", args);
    //eprintln!("raw input:{:?}", raw);

    //enum or match only
    let mut err_info = quote! {};
    match raw {
        syn::Item::Enum(ref input_enum) => {
            if let Err(e) = check_enum_order(input_enum) {
                let err = e.to_compile_error();
                err_info = quote! {#err}
            }
        }
        _ => {
            return syn::Error::new(Span::call_site(), "expected enum or match expression")
                .into_compile_error()
                .into();
        }
    }

    quote! {
        #err_info
        #raw
    }
    .into()
}

fn check_enum_order(input: &ItemEnum) -> Result<(), syn::Error> {
    let variants = &input.variants;
    let var_vec: Vec<_> = variants.iter().collect();
    for (i, src) in var_vec.iter().enumerate() {
        for des in &var_vec[..i] {
            if src.ident.to_string().to_lowercase() < des.ident.to_string().to_lowercase() {
                return Err(syn::Error::new(
                    src.span(),
                    &format!("{} should sort before {}", src.ident, des.ident),
                ));
            }
        }
    }

    Ok(())
}

struct MatchReplace {
    error: Option<syn::Error>,
}

impl syn::visit_mut::VisitMut for MatchReplace {
    fn visit_expr_match_mut(&mut self, node: &mut syn::ExprMatch) {
        let mut new_attr = Vec::new();
        let mut sort = false;
        for attr in &node.attrs {
            if attr.path().is_ident("sorted") {
                sort = true;
                continue;
            }
            new_attr.push(attr.clone());
        }
        node.attrs = new_attr;

        if sort {
            if let Err(e) = check_match_seq(node) {
                self.error = Some(e);
            }
        }

        syn::visit_mut::visit_expr_match_mut(self, node);
    }
}

#[proc_macro_attribute]
pub fn check(args: TokenStream, input: TokenStream) -> TokenStream {
    let _ = args;
    eprintln!("raw input in check:{}", input);
    let mut input = syn::parse_macro_input!(input as syn::ItemFn);
    //eprintln!("raw input in check:{:?}", input);
    let mut mr = MatchReplace { error: None };
    mr.visit_item_fn_mut(&mut input);
    let mut err_info = quote! {};
    if let Some(e) = mr.error {
        let ce = e.to_compile_error();
        err_info = quote! {#ce};
    }
    quote! {
        #err_info
        #input
    }
    .into()
}

fn check_match_seq(m: &syn::ExprMatch) -> Result<(), syn::Error> {
    eprintln!("ExprMatch:{:?}", m);
    for (i, arm) in m.arms.iter().enumerate() {
        //limit arm pattern format
        match arm.pat {
            syn::Pat::TupleStruct(_) => (),
            _ => return Err(syn::Error::new(arm.span(), "unsupported by #[sorted]")),
        }

        //check between prev arms
        for prev in &m.arms[..i] {
            if let syn::Pat::TupleStruct(syn::PatTupleStruct { path, .. }) = &arm.pat {
                if let syn::Pat::TupleStruct(syn::PatTupleStruct {
                    path: prev_path, ..
                }) = &prev.pat
                {
                    if path.get_ident().unwrap().to_string().to_uppercase()
                        < prev_path.get_ident().unwrap().to_string().to_uppercase()
                    {
                        return Err(syn::Error::new(
                            path.span(),
                            format!(
                                "{} should sort before {}",
                                path.get_ident().unwrap(),
                                prev_path.get_ident().unwrap()
                            ),
                        ));
                    }
                }
            }
        }
    }

    Ok(())
}
