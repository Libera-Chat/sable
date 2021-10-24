use super::*;
use quote::quote;
use syn::{
    parse_macro_input,
    parenthesized,
    braced,
    Token,
    Result,
    Ident,
    LitChar,
    LitInt,
    token,
    punctuated::Punctuated,
};
use syn::parse::{Parse, ParseStream};
use proc_macro2::Span;

struct ModeDef
{
    name: Ident,
    _paren: token::Paren,
    flag: LitInt,
    _comma: Token![,],
    modechar: LitChar
}

struct ModeSet
{
    name: Ident,
    _brace: token::Brace,
    items: Punctuated<ModeDef, Token![,]>
}

impl Parse for ModeDef
{
    fn parse(input: ParseStream) -> Result<Self>
    {
        let content;
        Ok(Self {
            name: input.parse()?,
            _paren: parenthesized!(content in input),
            flag: content.parse()?,
            _comma: content.parse()?,
            modechar: content.parse()?
        })
    }
}

impl Parse for ModeSet
{
    fn parse(input: ParseStream) -> Result<Self>
    {
        let content;
        Ok(Self{
            name: input.parse()?,
            _brace: braced!(content in input),
            items: content.parse_terminated(ModeDef::parse)?
        })
    }
}

pub fn modeflags(input: TokenStream) -> TokenStream
{
    let modes = parse_macro_input!(input as ModeSet);

    let name_one = Ident::new(&format!("{}Flag", modes.name), Span::call_site());
    let name_set = Ident::new(&format!("{}Set", modes.name), Span::call_site());
    let name_mask = Ident::new(&format!("{}Mask", modes.name), Span::call_site());

    let mut consts = Vec::new();
    let mut const_names = Vec::new();
    let mut mode_chars = Vec::new();
    let mut pairs = Vec::new();

    for item in modes.items
    {
        let name = item.name;
        let flag = item.flag;
        let modechar = item.modechar;

        consts.push(quote!(
            #name = #flag
        ));
        const_names.push(quote!(
            #name
        ));
        mode_chars.push(quote!(
            #modechar
        ));
        pairs.push(quote!(
            (#name_one::#flag, #modechar)
        ));
    }

    let num_items = consts.len();

    let output = quote!(
        #[derive(Debug,Clone,Copy,Eq,PartialEq)]
        pub enum #name_one
        {
            #( #consts ),*
        }

        #[derive(Debug,Clone,Copy,Eq,PartialEq,serde::Serialize,serde::Deserialize)]
        pub struct #name_set(u64);

        #[derive(Debug,Clone,Copy)]
        pub struct #name_mask(u64);

        impl #name_set
        {
            const ALL: [(#name_one, char); #num_items] = [ #( ( #name_one::#const_names, #mode_chars ) ),* ];

            pub fn is_set(&self, flag: #name_one) -> bool
            {
                (self.0 & flag as u64) != 0
            }

            pub fn is_empty(&self) -> bool
            {
                self.0 == 0
            }

            pub fn to_chars(&self) -> String
            {
                let mut s = String::new();
                for (f, c) in Self::ALL
                {
                    if (self.is_set(f))
                    {
                        s.push(c);
                    }
                }
                return s;
            }

            pub fn new() -> Self { Self(0) }

            pub fn all() -> [(#name_one, char); #num_items] { Self::ALL }

            pub fn char_for(flag: #name_one) -> char
            {
                for (f, c) in Self::ALL
                {
                    if f == flag { return c; }
                }
                panic!("Invalid flag value?");
            }

            pub fn flag_for(modechar: char) -> Option<#name_one>
            {
                for (f, c) in Self::ALL
                {
                    if c == modechar { return Some(f); }
                }
                None
            }
        }

        impl Default for #name_set
        {
            fn default() -> Self { Self(0) }
        }

        impl From<#name_one> for #name_set
        {
            fn from(x: #name_one) -> Self { Self(x as u64) }
        }

        impl std::ops::BitOr for #name_one
        {
            type Output = #name_set;
            fn bitor(self, rhs: Self) -> #name_set { #name_set(self as u64 | rhs as u64) }
        }

        impl std::ops::BitOr<#name_one> for #name_set
        {
            type Output = Self;
            fn bitor(self, rhs: #name_one) -> Self { Self(self.0 | rhs as u64 ) }
        }

        impl std::ops::BitOrAssign for #name_set
        {
            fn bitor_assign(&mut self, rhs: Self) { self.0 |= rhs.0; }
        }

        impl std::ops::BitOrAssign<#name_one> for #name_set
        {
            fn bitor_assign(&mut self, rhs: #name_one) { self.0 |= rhs as u64; }
        }

        impl std::ops::Not for #name_set
        {
            type Output = #name_mask;
            fn not(self) -> #name_mask { #name_mask(!self.0) }
        }

        impl std::ops::BitAnd<#name_mask> for #name_set
        {
            type Output = Self;
            fn bitand(self, rhs: #name_mask) -> Self { Self(self.0 & rhs.0) }
        }

        impl std::ops::BitAndAssign<#name_mask> for #name_set
        {
            fn bitand_assign(&mut self, rhs: #name_mask) { self.0 &= rhs.0 }
        }
    );

    output.into()
}