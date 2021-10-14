use super::*;
use quote::quote;
use syn::{
    parse_macro_input,
    braced,
    Token,
    Result,
    Ident,
    Expr,
    token,
    punctuated::Punctuated,
};
use syn::parse::{Parse, ParseStream};

struct HandlerList
{
    event_name: Ident,
    _arrow: Token![=>],
    _brace: token::Brace,
    handlers: Punctuated<Handler, Token![,]>
}

impl Parse for HandlerList
{
    fn parse(input: ParseStream) -> Result<Self> {
        let content;

        Ok(Self {
            event_name: input.parse()?,
            _arrow: input.parse()?,
            _brace: braced!(content in input),
            handlers: content.parse_terminated(Handler::parse)?,
        })
    }
}

struct Handler {
    event_type: Ident,
    _arrow: Token![=>],
    handler: Expr,
}

impl Parse for Handler
{
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            event_type: input.parse()?,
            _arrow: input.parse()?,
            handler: input.parse()?
        })
    }
}


pub fn dispatch_event(input: TokenStream, is_async: bool) -> TokenStream
{
    let handlers = parse_macro_input!(input as HandlerList);

    let mut cases = Vec::new();
    let event_name = handlers.event_name;

    let do_await = if is_async { Some(quote!(.await)) } else { None };

    for item in handlers.handlers
    {
        let event_type = item.event_type;
        let handler = item.handler;

        cases.push(quote!(
            crate::ircd::event::EventDetails::#event_type(detail) => {
                match <crate::ircd::event::#event_type as crate::ircd::event::DetailType>::Target::try_from(#event_name.target) {
                    Ok(id) => {
                        Ok(#handler (
                            id,
                            #event_name,
                            &detail
                        ) #do_await)
                    },
                    Err(e) => Err(e)
                }
            }
        ));
    }

    let out = quote!(
        {
            use std::convert::TryFrom;
            match &#event_name.details
            {
                #( #cases ),*
            }
        }
    );
    
    out.into()
}