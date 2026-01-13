use proc_macro::TokenStream;
use quote::quote;
use syn::spanned::Spanned;

pub fn generate(args: TokenStream, input: TokenStream) -> TokenStream {
    let _ = args;
    let _ = input;

    let input = syn::parse_macro_input!(input as syn::ItemStruct);
    let name = &input.ident;

    let syn::Fields::Named(fields) = &input.fields else {
        return syn::Error::new(input.span(), "Only FieldsNames are supported")
            .to_compile_error()
            .into();
    };

    let bit_size = calsize(fields);
    let bytes = quote! {(#bit_size)/8};
    let accs = gen_accs(name, fields);

    quote! {
        const TOTAL_BITS:usize = #bit_size;
        const CHECK_MOD : usize = TOTAL_BITS % 8;

        #[repr(C)]
        pub struct #name{
            data:[u8;#bytes],
        }

        impl #name{
            pub fn new()->Self{
                let _ : bitfield::checks::CheckMultipleOfEight<MultipleOfEight<CHECK_MOD>>;
                Self{
                    data:[0;#bytes],
                }
            }
            #accs
        }

    }
    .into()
}

fn gen_accs(_name: &syn::Ident, fields: &syn::FieldsNamed) -> proc_macro2::TokenStream {
    let mut offset = quote! {0usize};
    let mut output = quote! {};

    //getter, setter
    for f in &fields.named {
        if let Some(fid) = &f.ident {
            let len = get_bits(f);
            let setter_name = syn::Ident::new(&format!("set_{}", fid.to_string()), fid.span());
            let getter_name = syn::Ident::new(&format!("get_{}", fid.to_string()), fid.span());
            let ftype = &f.ty;

            //get body
            let get_body = quote! {
                let mut result = 0;
                for i in 0usize..#len{
                    let byte_index = (#offset + i)/8;
                    let byte_off = (#offset + i) %8;
                    let bit = (self.data[byte_index] >> byte_off) & 1;
                    result |= ((bit as <#ftype as bitfield::Specifier>::Base) << i);
                }
                result
            };

            //set body
            let set_body = quote! {
                for i in 0usize..#len{
                    let byte_index = (#offset + i)/8;
                    let byte_off = (#offset +i)%8;
                    let bit = (v>>i)&1;
                    let mut data = self.data[byte_index];
                    data &= !(1<<byte_off);
                    data |= ((bit as u8) << byte_off);
                    self.data[byte_index] = data;
                }
            };

            output.extend(quote! {
                pub fn #setter_name(&mut self, v:<#ftype as bitfield::Specifier>::Base){
                    #set_body
                }
                pub fn #getter_name(&self)-><#ftype as bitfield::Specifier>::Base{
                    #get_body
                }
            });

            offset = quote! {#offset + #len};
        }
    }
    output
}

fn calsize(fields: &syn::FieldsNamed) -> proc_macro2::TokenStream {
    fields.named.iter().fold(quote! {0usize}, |mut acc, f| {
        let bits = get_bits(f);
        let bits_size = quote! {
            +#bits
        };
        acc.extend(bits_size);
        acc
    })
}

fn get_bits(f: &syn::Field) -> proc_macro2::TokenStream {
    if let syn::Type::Path(path_type) = &f.ty {
        if let Some(t) = path_type.path.segments.last() {
            let enum_name = &t.ident;
            return quote! {
                #enum_name::BITS
            };
        }
    }

    quote! {0usize}
}
