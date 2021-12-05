use super::*;

use proc_macro2;
use proc_macro2::Span;
use quote::quote;
use syn::{
    parenthesized,
    parse_macro_input,
    Result,
    Block,
    Type,
    Ident,
    token,
};
use syn::parse::{Parse, ParseStream};

struct ValidatedDefnList
{
    items: Vec<ValidatedDefn>
}

struct ValidatedDefn
{
    name: Ident,
    _paren: token::Paren,
    utype: Type,
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
        let content;

        Ok(Self {
            name: input.parse()?,
            _paren: parenthesized!(content in input),
            utype: content.parse()?,
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
        let typename = def.utype;
        let body = def.body;

        let error = Ident::new(&format!("Invalid{}Error", name), Span::call_site());
        let error_str = format!("Invalid value for {}: {{0}}", name);

        out.extend(quote!(
            #[derive(Debug,Clone,Error)]
            #[error(#error_str)]
            pub struct #error(pub String);

            impl From<StringValidationError> for #error
            {
                fn from(e: StringValidationError) -> Self { Self(e.0) }
            }

            #[derive(Debug,Clone,Copy,PartialEq,Eq,Hash,PartialOrd,Ord,serde::Serialize,serde::Deserialize)]
            pub struct #name(#typename);

            impl #name
            {
                fn error(s: impl std::string::ToString) -> std::result::Result<(), #error>
                {
                    Err(#error (s.to_string()))
                }
            }

            impl crate::validated::Validated for #name
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

                fn from_str(arg: &str) -> Self::Result
                {
                    if let Ok(val) = <Self as Validated>::Underlying::try_from(arg)
                    {
                        Self::new(val)
                    }
                    else
                    {
                        Err(#error(arg.to_string()))
                    }
                }

                fn convert(arg: impl std::string::ToString) -> Self::Result
                {
                    <Self as std::convert::TryFrom<String>>::try_from(arg.to_string())
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

            impl std::convert::TryFrom<String> for #name
            {
                type Error = <Self as Validated>::Error;
                fn try_from(arg: String) -> Result<Self, Self::Error>
                {
                    if let Ok(val) = <Self as Validated>::Underlying::try_from(arg.as_str())
                    {
                        Self::new(val)
                    }
                    else
                    {
                        Err(#error(arg))
                    }
                }
            }

            impl std::convert::TryFrom<&str> for #name
            {
                type Error = <Self as Validated>::Error;
                fn try_from(arg: &str) -> Result<Self, Self::Error>
                {
                    Self::from_str(arg)
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
