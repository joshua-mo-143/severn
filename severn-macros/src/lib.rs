use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    parse_macro_input, Ident, ItemStruct, Lit, Token,
};

#[proc_macro_attribute]
pub fn severn(args: TokenStream, item: TokenStream) -> TokenStream {
    println!("{args:?}");
    let MyArgs {
        name_value,
        system_message_value,
        ..
    } = parse_macro_input!(args as MyArgs);
    let item = parse_macro_input!(item as ItemStruct);

    let ident = item.ident.clone();

    let quote = quote! {
        #item

        impl Agent for #ident {
            fn name(&self) -> String {
                #name_value.to_string()
            }

            fn system_message(&self) -> String {
                #system_message_value.to_string()
            }
        }
    };

    TokenStream::from(quote)
}

#[allow(dead_code)]
struct MyArgs {
    name_ident: Ident,
    equals_sign1: Token![=],
    name_value: String,
    comma: Token![,],
    system_message_ident: Ident,
    equals_sign2: Token![=],
    system_message_value: String,
}

impl Parse for MyArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        let name_ident = input.parse()?;
        let equals_sign1 = input.parse()?;
        let name_value = input.parse::<Lit>()?;

        let name_value = match name_value {
            Lit::Str(str) => str.value(),
            _ => return Err(lookahead.error()),
        };

        let comma = input.parse()?;

        let system_message_ident = input.parse()?;
        let equals_sign2 = input.parse()?;

        let system_message_value = input.parse::<Lit>()?;

        let system_message_value = match system_message_value {
            Lit::Str(str) => str.value(),
            _ => return Err(lookahead.error()),
        };

        Ok(Self {
            name_ident,
            equals_sign1,
            name_value,
            comma,
            system_message_ident,
            equals_sign2,
            system_message_value,
        })
    }
}
