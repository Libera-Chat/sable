use super::*;

use proc_macro2;
use proc_macro2::Span;
use quote::quote;
use syn::{
    parse_macro_input,
    Result,
    Token,
    Ident,
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
            contents: input.parse()?,
            is_sequential: input.parse()?,
            _semi: input.parse()?,
        })
    }
}

struct ObjectIdList
{
    items: Vec<ObjectIdDefn>
}

impl Parse for ObjectIdList
{
    fn parse(input: ParseStream) -> Result<Self>
    {
        let mut items = Vec::new();

        while !input.is_empty()
        {
            items.push(input.parse::<ObjectIdDefn>()?);
        }

        Ok(Self{items: items})
    }
}

pub fn object_ids(input: TokenStream) -> TokenStream
{
    let input = parse_macro_input!(input as ObjectIdList);
    object_ids_impl(input).into()
}

fn object_ids_impl(input: ObjectIdList) -> proc_macro2::TokenStream
{
    let mut output = proc_macro2::TokenStream::new();
    let mut enum_variants = Vec::new();
    let mut all_typenames = Vec::new();
    let mut generator_fields = Vec::new();
    let mut generator_methods = Vec::new();

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
            #[derive(PartialEq,Eq,Hash,Debug,Clone,Copy)]
            pub struct #id_typename #contents;

            impl #id_typename
            {
                pub fn new(#( #arg_list ),*) -> Self { Self(#( #arg_names ), *) }
            }
        ));

        if item.is_sequential.is_some()
        {
            // Generators hold all but the last field
            arg_types.pop();
            arg_names.pop();
            arg_list.pop();
    
            let field_numbers = (0..arg_types.len()).map(|i| syn::Index::from(i));
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
                }
            ));

            let generator_method_name = Ident::new(&format!("next_{}", &typename).to_ascii_lowercase(), Span::call_site());
            let generator_field_name = Ident::new(&format!("{}_generator_field", &typename), Span::call_site());
            let generator_access_name = Ident::new(&format!("{}_generator", &typename).to_ascii_lowercase(), Span::call_site());

            generator_methods.push(quote!(
                pub fn #generator_method_name (&self) -> #id_typename {
                    self. #generator_field_name . next()
                }

                pub fn #generator_access_name (&self) -> std::sync::Arc<#generator_typename> {
                    std::sync::Arc::clone(&self. #generator_field_name)
                }
            ));

            generator_fields.push(quote!(
                #generator_field_name : std::sync::Arc<#generator_typename>
            ))
        }

    }

    output.extend(quote!(
        #[derive(PartialEq,Eq,Hash,Debug,Clone,Copy)]
        pub enum ObjectId {
            #( #enum_variants ),*
        }

        /*
        #[derive(Default,Debug)]
        pub struct IdGenerator {
            #( #generator_fields ),*
        }

        impl IdGenerator {
            pub fn new() -> Self {
                Default::default()
            }

            #( #generator_methods )*
        }
        */
    ));

    output
}

#[cfg(test)]
mod test
{
    use super::*;
    
    #[test]
    fn simple_object_id()
    {
        let input = syn::parse_str::<ObjectIdList>("A: (i64,);").unwrap();
        let output = object_ids_impl(input);
        let s = format!("{}", output);
        assert_eq!(s, "a");
    }

    #[test]
    fn sequential_object_id()
    {
        let input = syn::parse_str::<ObjectIdList>("A: (i64,) sequential;").unwrap();
        let output = object_ids_impl(input);
        let s = format!("{}", output);
        assert_eq!(s, "a");
    }

    #[test]
    fn multiple_sequential_object_ids()
    {
        let input = syn::parse_str::<ObjectIdList>("A: (i64,) sequential; B: (i64, i64) sequential;").unwrap();
        let output = object_ids_impl(input);
        let s = format!("{}", output);
        assert_eq!(s, "a");
    }

}
