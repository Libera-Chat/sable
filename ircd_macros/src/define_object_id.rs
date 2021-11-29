use super::*;

use proc_macro2;
use proc_macro2::Span;
use quote::quote;
use syn::{
    parse_macro_input,
    Result,
    Token,
    Ident,
    Type,
    TypeTuple,
    token
};
//use syn::punctuated::Punctuated;
use syn::parse::{Parse, ParseStream};

mod kw {
    syn::custom_keyword!(sequential);
}

struct ObjectIdDefn
{
    typename: Ident,
    _colon: Token![:],
    contents: TypeTuple,
    is_sequential: Option<kw::sequential>,
    _semi: Token![;],
}

impl Parse for ObjectIdDefn
{
    fn parse(input: ParseStream) -> Result<Self>
    {
        Ok(Self {
            typename: input.parse()?,
            _colon: input.parse()?,
            contents: if input.peek(kw::sequential) { syn::parse_str("(ServerId,EpochId,LocalId)")? } else { input.parse()? },
            is_sequential: input.parse()?,
            _semi: input.parse()?,
        })
    }
}

struct ObjectIdList
{
    enum_name: Ident,
    _comma: Token![,],
    _brace: token::Brace,
    items: Vec<ObjectIdDefn>
}

impl Parse for ObjectIdList
{
    fn parse(input: ParseStream) -> Result<Self>
    {
        let mut items = Vec::new();
        let enum_name = input.parse()?;
        let _comma = input.parse()?;
        let content;
        let _brace = syn::braced!(content in input);

        while !content.is_empty()
        {
            items.push(content.parse::<ObjectIdDefn>()?);
        }

        Ok(Self {
            enum_name: enum_name,
            _comma: _comma,
            _brace: _brace,
            items: items
        })
    }
}

pub fn object_ids(input: TokenStream) -> TokenStream
{
    let input = parse_macro_input!(input as ObjectIdList);

    let mut output = proc_macro2::TokenStream::new();
    let enum_name = input.enum_name;
    let generator_name = Ident::new(&format!("{}Generator", enum_name), Span::call_site());
    let mut enum_variants = Vec::new();
    let mut all_typenames = Vec::new();
    let mut generator_fields = Vec::new();
    let mut generator_field_names = Vec::new();
    let mut generator_methods = Vec::new();
    let mut generator_initargs = Vec::new();

    for item in input.items
    {
        let typename = item.typename;
        let id_typename = Ident::new(&format!("{}Id", typename), Span::call_site());
        let contents = item.contents;

        let mut arg_types = Vec::new();
        let mut arg_names = Vec::new();
        let mut arg_list = Vec::new();

        for (argtype, n) in contents.elems.iter().zip(1..)
        {
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
                pub fn new(#( #arg_list ),*) -> Self { Self(#( #arg_names ), *) }
            }

            impl From<#id_typename> for #enum_name
            {
                fn from(id: #id_typename) -> Self {
                    Self::#typename(id)
                }
            }

            impl std::convert::TryFrom<#enum_name> for #id_typename
            {
                type Error = crate::id::WrongIdTypeError;

                fn try_from(id: #enum_name) -> Result<Self, crate::id::WrongIdTypeError> {
                    match id {
                        #enum_name::#typename(x) => Ok(x),
                        _ => Err(crate::id::WrongIdTypeError)
                    }
                }
            }
        ));

        if item.is_sequential.is_some()
        {
            // Generators hold all but the last field
            arg_types.pop();
            arg_names.pop();
            arg_list.pop();
    
            let field_numbers: Vec<_> = (0..arg_types.len()).map(|i| syn::Index::from(i)).collect();
            let counter_number = syn::Index::from(arg_types.len());

            let generator_typename = Ident::new(&format!("{}Generator", id_typename), Span::call_site());

            let maybe_comma = if arg_list.is_empty() { None } else { Some(token::Comma(Span::call_site())) };

            output.extend(quote!(
                #[derive(Debug)]
                pub struct #generator_typename(#( #arg_types ),* #maybe_comma std::sync::atomic::AtomicI64);

                impl #generator_typename
                {
                    pub fn new(#( #arg_list ),* #maybe_comma start: i64) -> Self {
                         Self(#( #arg_names ),* #maybe_comma std::sync::atomic::AtomicI64::new(start))
                    }

                    pub fn next(&self) -> #id_typename {
                        #id_typename::new(
                            #( self.#field_numbers ),* #maybe_comma 
                            self.#counter_number.fetch_add(1, std::sync::atomic::Ordering::SeqCst))
                    }

                    pub fn last(&self) -> #id_typename {
                        #id_typename::new(
                            #( self.#field_numbers ),* #maybe_comma
                            self.#counter_number.load(std::sync::atomic::Ordering::SeqCst))
                    }

                    pub fn update_to(&self, next: i64)
                    {
                        self.#counter_number.store(next, std::sync::atomic::Ordering::SeqCst);
                    }
                }
            ));

            let serverid_type = syn::parse::<Type>(quote!(ServerId).into()).unwrap();
            let epochid_type = syn::parse::<Type>(quote!(EpochId).into()).unwrap();

            if arg_types.len() == 2 && arg_types[0] == serverid_type && arg_types[1] == epochid_type
            {
                let generator_method_name = Ident::new(&format!("next_{}", &typename).to_ascii_lowercase(), Span::call_site());
                let generator_field_name = Ident::new(&format!("{}_generator_field", &typename), Span::call_site());

                generator_methods.push(quote!(
                    pub fn #generator_method_name (&self) -> #id_typename {
                        self. #generator_field_name . next()
                    }
                ));

                generator_fields.push(quote!(
                    #generator_field_name : #generator_typename
                ));

                generator_field_names.push(generator_field_name.clone());

                generator_initargs.push(quote!(
                    #generator_field_name: #generator_typename::new(server_id, epoch_id, 1)
                ));

                output.extend(quote!(
                    impl #generator_typename
                    {
                        pub fn set_epoch(&mut self, new_epoch: EpochId)
                        {
                            self.1 = new_epoch;
                        }
                    }
                ));
            }
        }
    }

    output.extend(quote!(
        #[derive(PartialEq,Eq,Hash,Debug,Clone,Copy,serde::Serialize,serde::Deserialize)]
        pub enum #enum_name {
            #( #enum_variants ),*
        }

        pub struct #generator_name {
            #( #generator_fields ),*
        }

        impl #generator_name {
            #( #generator_methods )*

            pub fn new(server_id: ServerId, epoch_id: EpochId) -> Self
            {
                Self {
                    #( #generator_initargs ),*
                }
            }

            pub fn set_epoch(&mut self, new_epoch: EpochId)
            {
                #( self. #generator_field_names .set_epoch(new_epoch); )*
            }
        }
    ));

    output.into()
}
