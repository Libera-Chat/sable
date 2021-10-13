use super::*;

use proc_macro2;
use quote::quote;
use syn::{
    parse_macro_input,
    Result,
    ItemStruct,
    Ident
};
use syn::parse::{Parse, ParseStream};

struct ItemStructList {
    items: Vec<ItemStruct>
}

impl Parse for ItemStructList {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut items = Vec::new();
        
        while ! input.is_empty() {
            items.push(input.parse::<ItemStruct>()?);
        }

        Ok(ItemStructList {
            items: items
        })
    }
}

pub fn event_details(input: TokenStream) -> TokenStream
{
    let items = parse_macro_input!(input as ItemStructList);
    
    let mut output = proc_macro2::TokenStream::new();
    let mut names = Vec::<Ident>::new();

    for item in &items.items
    {
        let name = &item.ident;
        names.push(name.clone());

        let attrs = &item.attrs;
        let fields = &item.fields;

        let defn = quote!(
            #( #attrs )*
            #[derive(Debug,Clone)]
            pub struct #name
            #fields
        );

        output.extend(defn);
    }

    output.extend(quote!(
        #[derive(Debug,Clone)]
        pub enum EventDetails {
            #( #names(#names) ),*
        }

        #(
            impl From<#names> for EventDetails
            {
                fn from(x: #names) -> Self { Self::#names(x) }
            }
        )*
    ));

    output.into()
}

pub fn target_type_attribute(attr: TokenStream, item: TokenStream) -> TokenStream
{
    let target_typename = parse_macro_input!(attr as Ident);
    let item = parse_macro_input!(item as ItemStruct);

    let name = &item.ident;

    quote!(
        #item

        impl crate::ircd::event::DetailType for #name
        {
            type Target = #target_typename;
        }
    ).into()
}