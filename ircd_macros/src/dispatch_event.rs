use super::*;
use quote::quote;
use syn::{
    parse_macro_input,
    braced,
    parenthesized,
    Token,
    Result,
    Ident,
    Expr,
    token,
    punctuated::Punctuated,
};
use syn::parse::{Parse, ParseStream};

struct ExtraArgList
{
    _paren: token::Paren,
    args: Punctuated<Expr, Token![,]>
}

struct HandlerList
{
    event_name: Ident,
    extra_args: Option<ExtraArgList>,
    _arrow: Token![=>],
    _brace: token::Brace,
    handlers: Punctuated<Handler, Token![,]>
}

impl Parse for ExtraArgList
{
    fn parse(input: ParseStream) -> Result<Self> {
        let content;

        Ok(Self {
            _paren: parenthesized!(content in input),
            args: content.parse_terminated(Expr::parse)?
        })
    }
}

impl Parse for HandlerList
{
    fn parse(input: ParseStream) -> Result<Self> {
        let content;

        Ok(Self {
            event_name: input.parse()?,
            extra_args: if input.peek(token::Paren) {
                    Some(input.parse()?)
                } else {
                    None
                },
            _arrow: input.parse()?,
            _brace: braced!(content in input),
            handlers: content.parse_terminated(Handler::parse)?,
        })
    }
}

enum EventType {
    Event(Ident),
    Any
}

struct Handler {
    event_type: EventType,
    _arrow: Token![=>],
    handler: Expr,
}

impl Parse for Handler
{
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Self {
            event_type: if input.peek(Token![_]) {
                input.parse::<Token![_]>()?;
                EventType::Any
            } else {
                EventType::Event(input.parse()?)
            },
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

    let extra_args = if let Some(extra) = handlers.extra_args {
        let list = extra.args;
        Some(quote!(, #list))
    } else {
        None
    };

    for item in handlers.handlers
    {
        let handler = item.handler;

        match item.event_type
        {
            EventType::Event(event_type) => {
                cases.push(quote!(
                    irc_network::event::EventDetails::#event_type(detail) => {
                        match <irc_network::event::#event_type as irc_network::event::DetailType>::Target::try_from(#event_name.target) {
                            Ok(id) => {
                                Ok(#handler (
                                    id,
                                    #event_name,
                                    &detail
                                    #extra_args
                                ) #do_await)
                            },
                            Err(e) => Err(e)
                        }
                    }
                ));
            },
            EventType::Any => {
                cases.push(quote!(
                    _ => { Ok( #handler ( #event_name ) ) }
                ))
            }
        }

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