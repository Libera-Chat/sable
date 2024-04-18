use super::*;
use proc_macro2::Span;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{
    braced, parenthesized, parse_macro_input, punctuated::Punctuated, token, Ident, LitChar,
    LitInt, Result, Token,
};

struct ModeDef {
    name: Ident,
    flag: LitInt,
    modechar: LitChar,
    prefixchar: Option<LitChar>,
}

struct ModeSet {
    name: Ident,
    _brace: token::Brace,
    items: Punctuated<ModeDef, Token![,]>,
}

impl Parse for ModeDef {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        let name = input.parse()?;
        let _: token::Paren = parenthesized!(content in input);
        let flag = content.parse()?;
        content.parse::<Token![,]>()?;
        let modechar = content.parse()?;
        let prefixchar = match content.parse::<Token![,]>() {
            Ok(_) => Some(content.parse()?),
            Err(_) => None,
        };
        Ok(Self {
            name,
            flag,
            modechar,
            prefixchar,
        })
    }
}

impl Parse for ModeSet {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        Ok(Self {
            name: input.parse()?,
            _brace: braced!(content in input),
            items: content.parse_terminated(ModeDef::parse)?,
        })
    }
}

pub fn mode_flags(input: TokenStream) -> TokenStream {
    let modes = parse_macro_input!(input as ModeSet);

    let name_one = Ident::new(&format!("{}Flag", modes.name), Span::call_site());
    let name_set = Ident::new(&format!("{}Set", modes.name), Span::call_site());
    let name_mask = Ident::new(&format!("{}Mask", modes.name), Span::call_site());

    let mut consts = Vec::new();
    let mut const_names = Vec::new();
    let mut mode_chars = Vec::new();
    let mut prefix_chars = Vec::new();
    let mut pairs = Vec::new();

    for item in modes.items {
        let name = item.name;
        let flag = item.flag;
        let modechar = item.modechar;
        let prefixchar = item.prefixchar;

        consts.push(quote!(
            #name = #flag
        ));
        const_names.push(quote!(
            #name
        ));
        mode_chars.push(quote!(
            #modechar
        ));
        if let Some(prefixchar) = prefixchar {
            prefix_chars.push(quote!(
                #prefixchar
            ));
        }
        pairs.push(quote!(
            (#name_one::#flag, #modechar)
        ));
    }

    assert!(
        prefix_chars.is_empty() || prefix_chars.len() == mode_chars.len(),
        "Got {} prefix chars, but there are {} mode chars",
        prefix_chars.len(),
        mode_chars.len(),
    );

    let num_items = consts.len();

    let mut output = quote!(
        #[derive(Debug,Clone,Copy,Eq,PartialEq)]
        pub enum #name_one
        {
            #( #consts ),*
        }

        impl #name_one
        {
            pub fn mode_char(self) -> char
            {
                match self {
                    #(
                        Self::#const_names => #mode_chars,
                    )*
                }
            }

            pub fn from_mode_char(modechar: char) -> Option<#name_one>
            {
                match modechar {
                    #(
                        #mode_chars => Some(Self::#const_names),
                    )*
                    _ => None,
                }
            }
        }

        #[derive(Debug,Clone,Copy,Eq,PartialEq,serde::Serialize,serde::Deserialize)]
        pub struct #name_set(u64);

        #[derive(Debug,Clone,Copy)]
        pub struct #name_mask(u64);

        impl #name_set
        {
            const ALL: [#name_one; #num_items] = [ #( #name_one::#const_names),* ];

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
                for v in Self::ALL
                {
                    if (self.is_set(v))
                    {
                        s.push(v.mode_char());
                    }
                }
                return s;
            }

            pub fn new() -> Self { Self(0) }

            pub fn all() -> [#name_one; #num_items] { Self::ALL }
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

        impl std::ops::BitOr for #name_set
        {
            type Output = Self;
            fn bitor(self, rhs: #name_set) -> Self { Self(self.0 | rhs.0) }
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

    if !prefix_chars.is_empty() {
        output.extend(quote!(
            impl #name_one
            {
                pub fn prefix_char(self) -> char
                {
                    match self {
                        #(
                            Self::#const_names => #prefix_chars,
                        )*
                    }
                }

                pub fn from_prefix_char(modechar: char) -> Option<#name_one>
                {
                    match modechar {
                        #(
                            #prefix_chars => Some(Self::#const_names),
                        )*
                        _ => None,
                    }
                }
            }

            impl #name_set
            {
                pub fn to_prefixes(&self) -> String
                {
                    let mut s = String::new();
                    for v in Self::ALL
                    {
                        if (self.is_set(v))
                        {
                            s.push(v.prefix_char());
                        }
                    }
                    return s;
                }

                /// [`to_prefixes`] fallback for clients without
                /// [`multi-prefix`](https://ircv3.net/specs/extensions/multi-prefix)
                pub fn to_highest_prefix(&self) -> Option<char>
                {
                    for v in Self::ALL
                    {
                        if (self.is_set(v))
                        {
                            return Some(v.prefix_char())
                        }
                    }
                    return None;
                }
            }
        ));
    }

    //panic!("{}", output);
    output.into()
}
