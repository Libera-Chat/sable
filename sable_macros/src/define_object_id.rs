use super::*;

use proc_macro2::Span;
use quote::quote;
use syn::{parse_macro_input, token, Ident, Result, Token, TypeTuple};
//use syn::punctuated::Punctuated;
use syn::parse::{Parse, ParseStream};

mod kw {
    syn::custom_keyword!(snowflake);
}

struct ObjectIdDefn {
    typename: Ident,
    _colon: Token![:],
    contents: TypeTuple,
    _is_snowflake: Option<kw::snowflake>,
    _semi: Token![;],
}

impl Parse for ObjectIdDefn {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            typename: input.parse()?,
            _colon: input.parse()?,
            contents: if input.peek(kw::snowflake) {
                syn::parse_str("(Snowflake,)")?
            } else {
                input.parse()?
            },
            _is_snowflake: input.parse()?,
            _semi: input.parse()?,
        })
    }
}

struct ObjectIdList {
    enum_name: Ident,
    generator_name: Option<Ident>,
    _brace: token::Brace,
    items: Vec<ObjectIdDefn>,
}

impl Parse for ObjectIdList {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut items = Vec::new();
        let enum_name = input.parse()?;
        let generator_name = if input.peek(token::Paren) {
            let content;
            let _paren: token::Paren = syn::parenthesized!(content in input);
            Some(content.parse()?)
        } else {
            None
        };

        let content2;
        let _brace = syn::braced!(content2 in input);

        while !content2.is_empty() {
            items.push(content2.parse::<ObjectIdDefn>()?);
        }

        Ok(Self {
            enum_name,
            generator_name,
            _brace,
            items,
        })
    }
}

pub fn object_ids(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ObjectIdList);

    let mut output = proc_macro2::TokenStream::new();
    let enum_name = input.enum_name;
    let generator_name = input.generator_name;
    let mut enum_variants = Vec::new();
    let mut all_typenames = Vec::new();

    for item in input.items {
        let typename = item.typename;
        let id_typename = Ident::new(&format!("{}Id", typename), Span::call_site());
        let contents = item.contents;

        let mut arg_types = Vec::new();
        let mut arg_names = Vec::new();
        let mut arg_list = Vec::new();

        for (argtype, n) in contents.elems.iter().zip(1..) {
            let argname = Ident::new(&format!("arg{}", n), Span::call_site());
            arg_types.push(argtype.clone());
            arg_names.push(argname.clone());
            arg_list.push(quote!(#argname: #argtype));
        }

        enum_variants.push(quote!(
            #typename(#id_typename)
        ));

        all_typenames.push(typename.clone());

        output.extend(quote!(
            #[derive(PartialEq,Eq,PartialOrd,Ord,Hash,Debug,Clone,Copy,serde::Serialize,serde::Deserialize)]
            pub struct #id_typename #contents;

            impl #id_typename
            {
                pub const fn new(#( #arg_list ),*) -> Self { Self(#( #arg_names ), *) }
            }

            impl From<#id_typename> for #enum_name
            {
                fn from(id: #id_typename) -> Self {
                    Self::#typename(id)
                }
            }

            impl std::convert::TryFrom<#enum_name> for #id_typename
            {
                type Error = WrongIdTypeError;

                fn try_from(id: #enum_name) -> Result<Self, WrongIdTypeError> {
                    match id {
                        #enum_name::#typename(x) => Ok(x),
                        _ => Err(WrongIdTypeError)
                    }
                }
            }
        ));

        // If it's a single type, generate a From/Deref impl
        if contents.elems.len() == 1 {
            let inner_type = contents.elems.first().unwrap();

            output.extend(quote!(
                impl std::convert::From<#inner_type> for #id_typename {
                    fn from(val: #inner_type) -> Self { Self(val) }
                }

                impl Deref for #id_typename {
                    type Target = #inner_type;
                    fn deref(&self) -> &#inner_type { &self.0 }
                }
            ));
        }
    }

    output.extend(quote!(
        #[derive(PartialEq,Eq,Hash,Debug,Clone,Copy,serde::Serialize,serde::Deserialize)]
        pub enum #enum_name {
            #( #enum_variants ),*
        }
    ));

    if generator_name.is_some() {
        output.extend(quote!(
            #[derive(PartialEq,Eq,PartialOrd,Ord,Hash,Debug,Clone,Copy,serde::Serialize,serde::Deserialize)]
            #[serde(transparent)]
            pub struct Snowflake(u64);

            impl Snowflake {
                pub fn server(&self) -> ServerId {
                    ServerId(((self.0 >> 12) & 0x3ff) as u16)
                }

                pub fn timestamp(&self) -> u64 {
                    self.0 >> 22
                }

                pub fn serial(&self) -> u16 {
                    (self.0 & 0xfff) as u16
                }

                pub const ZERO: Self = Self(0);

                pub fn from_parts(server: impl Into<ServerId>, timestamp: u64, serial: u16) -> Self {
                    Self(timestamp << 22 | (server.into().0 as u64 & 0x3ff) << 12 | (serial as u64 & 0xfff))
                }
            }

            impl AsRef<u64> for Snowflake {
                fn as_ref(&self) -> &u64 {
                    &self.0
                }
            }

            #[derive(Debug,serde::Serialize,serde::Deserialize)]
            pub struct #generator_name {
                server_id: u16,     // Actually 10 bits, enforced in new
                serial: std::sync::atomic::AtomicU16,
                                    // Format requires 12 bits, but we let this be 16 and ignore the top 4.
                                    // See comment in `next()`
            }

            impl #generator_name {
                // Jan 1, 2020, as milliseconds since the unix epoch.
                // I'd like this to be a chrono::DateTime but those are difficult to construct
                // const-ly from a unix timestamp.
                const SNOWFLAKE_EPOCH: u64 = 1577836800_000;

                pub fn new(server_id: ServerId) -> Self
                {
                    Self {
                        server_id: server_id.0 & 0x3ff, // 10-bit server ID
                        serial: 0.into(),
                    }
                }

                pub fn next<T: From<Snowflake>>(&self) -> T {
                    // `self.serial` is 16 bits, with fetch_add wrapping on overflow.
                    // We need 12-bit wrapping, which we can have by just letting it wrap itself and
                    // discarding the top bits. The output will wrap 16 times before the actual atomic
                    // does, but the visible behaviour is the same.
                    let next_serial = self.serial.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    let next_serial = next_serial & 0xfff; // 12 bits only.

                    let timestamp = (chrono::Utc::now().timestamp_millis() as u64 - Self::SNOWFLAKE_EPOCH);
                    let timestamp = timestamp & 0x3ff_ffff_ffff; // 42 bits

                    Snowflake(timestamp << 22 | (self.server_id as u64) << 12 | (next_serial as u64)).into()
                }
            }
        ));
    }

    output.into()
}
