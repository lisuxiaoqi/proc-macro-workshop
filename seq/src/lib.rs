use proc_macro::TokenStream;
use proc_macro2::{Span, TokenTree};
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;

#[proc_macro]
pub fn seq(input: TokenStream) -> TokenStream {
    let raw = proc_macro2::TokenStream::from(input);
    //for tt in raw {
    //    eprintln!("tt:{}", tt.to_string());
    //}

    //parse N, start, end
    let mut raw_iter = raw.into_iter();
    let var_ident = if let Some(TokenTree::Ident(n)) = raw_iter.next() {
        n
    } else {
        return syn::Error::new(Span::call_site(), "Expected loop variables")
            .to_compile_error()
            .into();
    };

    eprintln!("parsed loop variable:{}", var_ident);

    if let Some(TokenTree::Ident(ref kw)) = raw_iter.next() {
        if kw != "in" {
            return syn::Error::new(kw.span(), "Keyword in expected")
                .to_compile_error()
                .into();
        } else {
            eprintln!("parsed keyword in");
        }
    } else {
        return syn::Error::new(Span::call_site(), "Keyword in expected")
            .to_compile_error()
            .into();
    }

    let start = if let Some(TokenTree::Literal(lit)) = raw_iter.next() {
        lit.to_string().parse::<usize>().unwrap()
    } else {
        return syn::Error::new(Span::call_site(), "Keyword in expected")
            .to_compile_error()
            .into();
    };
    eprintln!("parsed start index:{}", start);

    for _ in 0..2 {
        match raw_iter.next() {
            Some(TokenTree::Punct(p)) if p.as_char() == '.' => {
                eprintln!("matched punctuation .");
            }
            _ => {
                return syn::Error::new(Span::call_site(), "Expected .. punctuation")
                    .into_compile_error()
                    .into();
            }
        }
    }

    let end = if let Some(TokenTree::Literal(lit)) = raw_iter.next() {
        lit.to_string().parse::<usize>().unwrap()
    } else {
        return syn::Error::new(Span::call_site(), "Keyword in expected")
            .to_compile_error()
            .into();
    };
    eprintln!("parsed end index:{}", end);

    //get body Token
    let Some(TokenTree::Group(g)) = raw_iter.next() else {
        return syn::Error::new(Span::call_site(), "Body group is expected")
            .to_compile_error()
            .into();
    };

    let mut body_tokens = g.stream();
    eprintln!("parsed body:{:?}", body_tokens);

    //locate inner repeat range
    let output = process_group(&mut body_tokens, &var_ident, start, end);
    eprintln!("expanded output:{}", output);

    // do code replace
    quote! {#output}.into()
}

fn process_group(
    body: &mut proc_macro2::TokenStream,
    var_ident: &proc_macro2::Ident,
    start: usize,
    end: usize,
) -> proc_macro2::TokenStream {
    eprintln!("process_group:{var_ident}, {start}, {end}, {body}");
    let mut output = proc_macro2::TokenStream::new();
    let sub_range = false;

    if !sub_range {
        for n in start..end {
            let rep = replace_stream(body, var_ident, n);
            output.extend(quote! {#rep});
        }
    }

    output
}

fn replace_stream(
    body: &proc_macro2::TokenStream,
    var_ident: &proc_macro2::Ident,
    val: usize,
) -> proc_macro2::TokenStream {
    let mut output = proc_macro2::TokenStream::new();
    eprintln!("\treplace_stream, body:{}", body);

    for it in body.clone() {
        match it {
            TokenTree::Ident(ident) => {
                eprintln!("\t\t replace_stream, get ident:{}", ident);
                if &ident == var_ident {
                    eprintln!("\t\t ident match");
                    let mut val_id = proc_macro2::Literal::usize_unsuffixed(val);
                    val_id.set_span(ident.span());
                    output.extend(quote! {#val_id});
                } else {
                    //not the var_ident
                    eprintln!("\t\t ident don't  match");
                    output.extend(quote! {#ident});
                }
            }
            TokenTree::Group(g) => {
                let inner = replace_stream(&g.stream(), var_ident, val);
                let mut rg = proc_macro2::Group::new(g.delimiter(), inner);
                rg.set_span(g.span());
                output.extend(quote! {#rg});
            }
            other => {
                output.extend(quote! {#other});
            }
        }
    }

    eprintln!("\treplace_stream, output:{}", output);
    output
}
