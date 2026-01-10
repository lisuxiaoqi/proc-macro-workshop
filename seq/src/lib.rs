use proc_macro::TokenStream;
use proc_macro2::{Span, TokenTree};
use quote::quote;

#[proc_macro]
pub fn seq(input: TokenStream) -> TokenStream {
    let raw = proc_macro2::TokenStream::from(input);
    eprintln!("seq input, raw:{0}\n, debug:{0:?}", raw);

    //parse N, start, end
    let mut raw_iter = raw.into_iter().peekable();
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

    //try to match = range
    let mut inclusive = false;
    if let Some(TokenTree::Punct(p)) = raw_iter.peek() {
        if p.as_char() == '=' {
            inclusive = true;
            raw_iter.next();
        }
    }

    let mut end = match raw_iter.next() {
        Some(TokenTree::Literal(lit)) => lit.to_string().parse::<usize>().unwrap(),
        Some(TokenTree::Group(g)) => {
            let stream = g.stream();
            if let Some(TokenTree::Literal(macro_lit)) = &stream.into_iter().next() {
                macro_lit.to_string().parse::<usize>().unwrap()
            } else {
                return syn::Error::new(Span::call_site(), "invalid range value")
                    .to_compile_error()
                    .into();
            }
        }
        _ => {
            return syn::Error::new(Span::call_site(), "invalid range value")
                .to_compile_error()
                .into();
        }
    };

    if inclusive {
        end += 1;
    }
    eprintln!("parsed end index:{}", end);

    //get body Token
    let Some(TokenTree::Group(g)) = raw_iter.next() else {
        return syn::Error::new(Span::call_site(), "Body group is expected")
            .to_compile_error()
            .into();
    };

    let body_tokens = g.stream();
    //eprintln!("parsed body:{:?}", body_tokens);

    //locate inner repeat range
    let output = process_group(&body_tokens, &var_ident, start, end);
    eprintln!("expanded output:{}", output);

    // do code replace
    quote! {#output}.into()
}

fn process_group(
    body: &proc_macro2::TokenStream,
    var_ident: &proc_macro2::Ident,
    start: usize,
    end: usize,
) -> proc_macro2::TokenStream {
    eprintln!("process_group:{var_ident}, {start}, {end}, {body}");
    //try to parse sub range
    let (sub_range, rangeout) = parse_repeat_section(body, var_ident, start, end);
    if sub_range {
        return rangeout;
    }

    // no subrange, parse whole body
    let mut output = proc_macro2::TokenStream::new();
    for n in start..end {
        let sout = replace_stream(body, var_ident, n);
        output.extend(quote! {#sout});
    }

    output
}

fn parse_repeat_section(
    body: &proc_macro2::TokenStream,
    var_ident: &proc_macro2::Ident,
    start: usize,
    end: usize,
) -> (bool, proc_macro2::TokenStream) {
    let mut output = proc_macro2::TokenStream::new();
    let mut range = false;
    let tt_vec: Vec<_> = body.clone().into_iter().collect();
    let mut i = 0;
    while i < tt_vec.len() {
        if i + 2 < tt_vec.len() {
            if let (TokenTree::Punct(prefix), TokenTree::Group(g), TokenTree::Punct(suffix)) =
                (&tt_vec[i], &tt_vec[i + 1], &tt_vec[i + 2])
            {
                if prefix.as_char() == '#' && suffix.as_char() == '*' {
                    eprintln!("Matched group range:{}", g);
                    range = true;
                    for n in start..end {
                        let sout = replace_stream(&g.stream(), var_ident, n);
                        output.extend(quote! {#sout});
                    }
                    i += 3;
                    continue;
                }
            }
        }

        match &tt_vec[i] {
            TokenTree::Group(g) => {
                let (sub, gout) = parse_repeat_section(&g.stream(), var_ident, start, end);
                range = sub;
                let mut new_group = proc_macro2::Group::new(g.delimiter(), gout);
                new_group.set_span(g.span());
                output.extend(quote! {#new_group});
            }
            other => {
                output.extend(quote! {#other});
            }
        }

        i += 1;
    }

    (range, output)
}

fn replace_stream(
    body: &proc_macro2::TokenStream,
    var_ident: &proc_macro2::Ident,
    val: usize,
) -> proc_macro2::TokenStream {
    let mut output = proc_macro2::TokenStream::new();
    let ident_out = replace_ident_stream(body, var_ident, val);
    let out = replace_lit_stream(&ident_out, var_ident, val);
    output.extend(quote! {#out});

    output
}

fn replace_lit_stream(
    body: &proc_macro2::TokenStream,
    var_ident: &proc_macro2::Ident,
    val: usize,
) -> proc_macro2::TokenStream {
    let mut output = proc_macro2::TokenStream::new();
    //eprintln!("replace_lit_stream, body:{}", body);

    for it in body.clone() {
        match it {
            TokenTree::Ident(ident) => {
                //eprintln!("\t\t replace_stream, get ident:{}", ident);
                //replace literal
                if &ident == var_ident {
                    //eprintln!("\t\t ident value match");
                    let mut val_id = proc_macro2::Literal::usize_unsuffixed(val);
                    val_id.set_span(ident.span());
                    output.extend(quote! {#val_id});
                } else {
                    //not the var_ident
                    //eprintln!("\t\t ident don't  match");
                    output.extend(quote! {#ident});
                }
            }
            TokenTree::Group(g) => {
                let inner = replace_lit_stream(&g.stream(), var_ident, val);
                let mut rg = proc_macro2::Group::new(g.delimiter(), inner);
                rg.set_span(g.span());
                output.extend(quote! {#rg});
            }
            other => {
                output.extend(quote! {#other});
            }
        }
    }

    //eprintln!("replace_lit_stream, output:{}", output);
    output
}

fn replace_ident_stream(
    body: &proc_macro2::TokenStream,
    var_ident: &proc_macro2::Ident,
    val: usize,
) -> proc_macro2::TokenStream {
    let mut output = proc_macro2::TokenStream::new();
    //eprintln!("replace_ident_stream, body:{}", body);
    let tt_vec: Vec<_> = body.clone().into_iter().collect();

    let mut i = 0;
    while i < tt_vec.len() {
        if i + 2 < tt_vec.len() {
            if let (TokenTree::Ident(prefix), TokenTree::Punct(punct), TokenTree::Ident(kw)) =
                (&tt_vec[i], &tt_vec[i + 1], &tt_vec[i + 2])
            {
                //matched ~N
                if punct.as_char() == '~' && kw == var_ident {
                    //eprintln!("matched identity:{}{}{}", prefix, punct, kw);

                    let old = format!("{}{}{}", prefix, punct, kw);
                    //eprintln!("old string with identity:{}", old);
                    let replaced = old.replace(&format!("~{}", kw), &format!("{}", val));
                    //eprintln!("new string with identity:{}", replaced);
                    let rep_ident = proc_macro2::Ident::new(&replaced, prefix.span());
                    output.extend(quote! {#rep_ident});
                    i = i + 3;
                    continue;
                }
            }
        }

        match &tt_vec[i] {
            TokenTree::Group(g) => {
                let gout = replace_ident_stream(&g.stream(), var_ident, val);
                let mut newgroup = proc_macro2::Group::new(g.delimiter(), gout);
                newgroup.set_span(g.span());
                output.extend(quote! {#newgroup});
            }
            other => {
                //eprintln!("\tmatched non group:{}", other);
                output.extend(quote! {#other});
            }
        }

        i += 1;
    }

    //eprintln!("replace_ident_stream, output:{}", output);
    output
}
