use proc_macro::TokenStream;
mod bitfield;
mod gen_type;

#[proc_macro_attribute]
pub fn bitfield(args: TokenStream, input: TokenStream) -> TokenStream {
    bitfield::generate(args, input)
}

#[proc_macro]
pub fn expand_types(input: TokenStream) -> TokenStream {
    gen_type::generate(input)
}
