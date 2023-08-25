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
    _paren: token::Paren,
    flag: LitInt,
    _comma: Token![,],
    modechars: Punctuated<LitChar, Token![,]>,
}

struct ModeSet {
    name: Ident,
    _brace: token::Brace,
    items: Punctuated<ModeDef, Token![,]>,
}

impl Parse for ModeDef {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        Ok(Self {
            name: input.parse()?,
            _paren: parenthesized!(content in input),
            flag: content.parse()?,
            _comma: content.parse()?,
            modechars: content.parse_terminated(LitChar::parse)?,
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
    let mut pairs = Vec::new();

    let mut mode_char_types = proc_macro2::TokenStream::new();
    let num_chartypes = modes.items[0].modechars.len();
    for _c in 0..num_chartypes {
        mode_char_types.extend(quote!(char,));
    }

    for item in modes.items {
        let name = item.name;
        let flag = item.flag;
        let modechars = item.modechars;

        consts.push(quote!(
            #name = #flag
        ));
        const_names.push(quote!(
            #name
        ));
        mode_chars.push(quote!(
            #modechars
        ));
        pairs.push(quote!(
            (#name_one::#flag, #modechars)
        ));
    }

    let num_items = consts.len();

    let mut output = quote!(
        #[derive(Debug,Clone,Copy,Eq,PartialEq)]
        pub enum #name_one
        {
            #( #consts ),*
        }

        impl #name_one
        {
            pub fn to_char(self) -> char
            {
                #name_set::char_for(self)
            }
        }

        #[derive(Debug,Clone,Copy,Eq,PartialEq,serde::Serialize,serde::Deserialize)]
        pub struct #name_set(u64);

        #[derive(Debug,Clone,Copy)]
        pub struct #name_mask(u64);

        impl #name_set
        {
            const ALL: [(#name_one, #mode_char_types); #num_items] = [ #( ( #name_one::#const_names, #mode_chars ) ),* ];

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
                    if (self.is_set(v.0))
                    {
                        s.push(v.1);
                    }
                }
                return s;
            }

            pub fn new() -> Self { Self(0) }

            pub fn all() -> [(#name_one, #mode_char_types); #num_items] { Self::ALL }

            pub fn char_for(flag: #name_one) -> char
            {
                for v in Self::ALL
                {
                    if v.0 == flag { return v.1; }
                }
                panic!("Invalid flag value?");
            }

            pub fn flag_for(modechar: char) -> Option<#name_one>
            {
                for v in Self::ALL
                {
                    if v.1 == modechar { return Some(v.0); }
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

    if num_chartypes > 1 {
        output.extend(quote!(
            impl #name_one
            {
                pub fn to_prefix(self) -> char
                {
                    #name_set::prefix_for(self)
                }
            }

            impl #name_set
            {
                pub fn to_prefixes(&self) -> String
                {
                    let mut s = String::new();
                    for v in Self::ALL
                    {
                        if (self.is_set(v.0))
                        {
                            s.push(v.2);
                        }
                    }
                    return s;
                }

                pub fn prefix_for(flag: #name_one) -> char
                {
                    for v in Self::ALL
                    {
                        if v.0 == flag { return v.2; }
                    }
                    panic!("Invalid flag value?");
                }

                pub fn flag_for_prefix(modechar: char) -> Option<#name_one>
                {
                    for v in Self::ALL
                    {
                        if v.2 == modechar { return Some(v.0); }
                    }
                    None
                }

            }
        ));
    }

    //panic!("{}", output);
    output.into()
}
