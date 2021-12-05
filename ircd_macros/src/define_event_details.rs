use super::*;

use proc_macro2;
use quote::quote;
use syn::{
    parse_macro_input,
    braced,
    Result,
    Attribute,
    ItemStruct,
    Ident,
    Token,
    token,
};
use syn::parse::{Parse, ParseStream};

struct DefinitionList {
    attrs: Vec<Attribute>,
    enum_name: Ident,
    _arrow: Token![=>],
    _brace: token::Brace,
    items: ItemStructList,
}

struct ItemStructList {
    items: Vec<ItemStruct>
}

impl Parse for DefinitionList {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        Ok(Self{
            attrs: input.call(Attribute::parse_outer)?,
            enum_name: input.parse()?,
            _arrow: input.parse()?,
            _brace: braced!(content in input),
            items: content.parse()?,
        })
    }
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
    let input = parse_macro_input!(input as DefinitionList);
    let attrs = input.attrs;
    let enum_name = input.enum_name;
    let items = input.items;
    
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
            #[derive(Debug,Clone,serde::Serialize,serde::Deserialize)]
            pub struct #name
            #fields
        );

        output.extend(defn);
    }

    output.extend(quote!(
        #( #attrs )*
        #[derive(Debug,Clone,serde::Serialize,serde::Deserialize)]
        pub enum #enum_name {
            #( #names(#names) ),*
        }

        #(
            impl From<#names> for #enum_name
            {
                fn from(x: #names) -> Self { Self::#names(x) }
            }

            impl std::convert::TryFrom<#enum_name> for #names
            {
                type Error = WrongEventTypeError;
                fn try_from(e: #enum_name) -> Result<Self, WrongEventTypeError>
                {
                    match e {
                        #enum_name::#names(x) => Ok(x),
                        _ => Err(WrongEventTypeError)
                    }
                }
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

        impl crate::event::DetailType for #name
        {
            type Target = #target_typename;
        }
    ).into()
}