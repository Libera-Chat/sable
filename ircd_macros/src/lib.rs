extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2;
use quote::quote;
use syn::{parse_macro_input, Result, Token, ItemStruct, Ident};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;

struct ItemStructList {
    items: Punctuated<ItemStruct, Token![;]>
}

impl Parse for ItemStructList {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(ItemStructList {
            items: input.parse_terminated(ItemStruct::parse)?
        })
    }
}


#[proc_macro]
pub fn event_details(input: TokenStream) -> TokenStream
{
    let items = parse_macro_input!(input as ItemStructList);
    
    let mut output = proc_macro2::TokenStream::new();
    let mut names = Vec::<Ident>::new();

    for item in &items.items
    {
        let name = item.ident.clone();
        names.push(name);

        let defn = quote!(
            #[derive(Debug,Clone)]
            pub #item
        );

        output.extend(defn);
    }

    output.extend(quote!(
        #[derive(Debug,Clone)]
        pub enum EventDetails {
            #( #names(#names) ),*
        }
    ));

    output.into()
}