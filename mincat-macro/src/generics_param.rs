use quote::{format_ident, quote};
use syn::{
    parse::{Parse, ParseStream, Result},
    parse_macro_input,
    token::Comma,
    Ident, LitInt,
};

struct Param {
    macro_ident: Ident,
    max: usize,
    ident: Ident,
}

impl Parse for Param {
    fn parse(input: ParseStream) -> Result<Self> {
        let macro_ident = input.parse::<Ident>()?;
        input.parse::<Comma>()?;
        let max = input.parse::<LitInt>()?.base10_parse()?;
        input.parse::<Comma>()?;
        let ident = input.parse::<Ident>()?;

        Ok(Param {
            macro_ident,
            max,
            ident,
        })
    }
}

pub fn generate(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as Param);
    let mut ident_tuples = Vec::with_capacity(input.max);
    for i in 0..input.max {
        let ident = format_ident!("{}{}", input.ident, i);
        ident_tuples.push(quote! {
            #ident
        });
    }
    let macro_ident = &input.macro_ident;
    let invocations = (0..input.max).map(|i| {
        let ident_tuples = &ident_tuples[..i];
        quote! {
            #macro_ident!(#(#ident_tuples),*);
        }
    });

    quote!(
        #(
            #invocations
        )*
    )
    .into()
}
