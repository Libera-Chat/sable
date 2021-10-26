use super::*;

use proc_macro2;
use proc_macro2::Span;
use quote::quote;
use syn::{
    parse_macro_input,
    Result,
    Block,
    Ident,
};
use syn::parse::{Parse, ParseStream};

struct ValidatedDefnList
{
    items: Vec<ValidatedDefn>
}

struct ValidatedDefn
{
    name: Ident,
    body: Block,
}

impl Parse for ValidatedDefnList
{
    fn parse(input: ParseStream) -> Result<Self>
    {
        let mut out = Vec::new();
        while !input.is_empty()
        {
            out.push(ValidatedDefn::parse(input)?);
        }
        Ok(Self{items: out})
    }
}

impl Parse for ValidatedDefn
{
    fn parse(input: ParseStream) -> Result<Self>
    {
        Ok(Self {
            name: input.parse()?,
            body: input.parse()?,
        })
    }
}

pub fn define_validated(input: TokenStream) -> TokenStream
{
    let input = parse_macro_input!(input as ValidatedDefnList);

    let mut out = proc_macro2::TokenStream::new();

    for def in input.items
    {
        let name = def.name;
        let typename = quote!(String);
        let body = def.body;

        let error = Ident::new(&format!("Invalid{}Error", name), Span::call_site());
        let error_str = format!("Invalid value for {}: {{0}}", name);

        out.extend(quote!(
            #[derive(Debug,Clone,Error)]
            #[error(#error_str)]
            pub struct #error(pub String);

            #[derive(Debug,Clone,PartialEq,serde::Serialize,serde::Deserialize)]
            pub struct #name(#typename);

            impl #name
            {
                fn error(s: impl std::string::ToString) -> std::result::Result<(), #error>
                {
                    Err(#error (s.to_string()))
                }
            }

            impl crate::ircd::validated::Validated for #name
            {
                type Underlying = #typename;
                type Error = #error;
                type Result = std::result::Result<#name, #error>;

                fn validate(value: &#typename) -> std::result::Result<(), <Self as Validated>::Error>
                #body

                fn new(arg: #typename) -> Self::Result
                {
                    Self::validate(&arg)?;
                    Ok(Self(arg))
                }

                fn value(&self) -> &Self::Underlying
                {
                    &self.0
                }
            }

            impl std::convert::TryFrom<#typename> for #name
            {
                type Error = <Self as Validated>::Error;
                fn try_from(arg: #typename) -> Result<Self, Self::Error>
                {
                    Self::new(arg)
                }
            }

            impl Into<#typename> for #name
            {
                fn into(self) -> #typename
                {
                    self.0
                }
            }

            impl std::fmt::Display for #name where #typename: std::fmt::Display
            {
                fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result
                {
                    self.0.fmt(f)
                }
            }

            impl<T> std::cmp::PartialEq<T> for #name where #typename: std::cmp::PartialEq<T>
            {
                fn eq(&self, other: &T) -> bool
                {
                    self.0.eq(other)
                }
            }
        ));
    }

    out.into()
}
