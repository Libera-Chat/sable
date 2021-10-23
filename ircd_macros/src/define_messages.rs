use super::*;
use quote::quote;
use syn::{
    parse_macro_input,
    braced,
    parenthesized,
    Token,
    Type,
    Result,
    Ident,
    Expr,
    LitStr,
    LitInt,
    token,
    punctuated::Punctuated,
};
use syn::parse::{Parse, ParseStream};
use proc_macro2::Span;

mod kw
{
    syn::custom_keyword!(source);
    syn::custom_keyword!(target);
}

enum MessageArg
{
    Source(kw::source),
    Target(kw::target),
    Arg(MessageArgDefn),
}

struct MessageArgDefn
{
    name: Ident,
    _colon: Token![:],
    typename: Type,
    _dot: Option<Token![.]>,
    expr: Option<Expr>
}

struct MessageDefn
{
    is_numeric: bool,
    name: String,
    typename: Ident,
    aliases: Option<Punctuated<Ident, Token![,]>>,
    _arrow1: Token![=>],
    _brace: token::Brace,
      _paren2: token::Paren,
        args: Punctuated<MessageArg, Token![,]>,
      _arrow2: Token![=>],
      value: LitStr,
}

struct MessageDefnList
{
    messages: Punctuated<MessageDefn, Token![,]>
}

impl Parse for MessageArg
{
    fn parse(input: ParseStream) -> Result<Self>
    {
        let lookahead = input.lookahead1();
        if lookahead.peek(kw::source)
        {
            Ok(Self::Source(input.parse()?))
        }
        else if lookahead.peek(kw::target)
        {
            Ok(Self::Target(input.parse()?))
        }
        else
        {
            Ok(Self::Arg(input.parse()?))
        }
    }
}

impl Parse for MessageArgDefn
{
    fn parse(input: ParseStream) -> Result<Self>
    {
        let name = input.parse::<Ident>()?;
        let _colon = input.parse::<Token![:]>()?;
        let typename = input.parse::<Type>()?;
        let _dot = input.parse::<Option<Token![.]>>()?;
        let expr = if _dot.is_some() { Some(input.parse::<Expr>()?) } else { None };

        Ok(Self { name: name, _colon: _colon, typename: typename, _dot: _dot, expr: expr })
    }
}

impl Parse for MessageDefn
{
    fn parse(input: ParseStream) -> Result<Self>
    {
        let content1;
        let content2;
        let content3;

        let (is_numeric, name, typename) = if let Ok(i) = input.parse::<LitInt>() {
            (true, i.to_string(), Ident::new(&format!("Numeric{}", i), Span::call_site()))
        } else {
            let ident: Ident = input.parse()?;
            (false, ident.to_string(), ident)
        };

        let aliases = if input.peek(token::Paren) {
            let _paren = parenthesized!(content1 in input);
            Some(content1.parse_terminated(Ident::parse)?)
        } else {
            None
        };

        Ok(MessageDefn {
            is_numeric: is_numeric,
            name: name,
            typename: typename,
            aliases: aliases,
            _arrow1: input.parse()?,
            _brace: braced!(content2 in input),
            _paren2: parenthesized!(content3 in content2),
            args: content3.parse_terminated(MessageArg::parse)?,
            _arrow2: content2.parse()?,
            value: content2.parse()?
        })
    }
}

impl Parse for MessageDefnList
{
    fn parse(input: ParseStream) -> Result<Self>
    {
        Ok(Self {
            messages: input.parse_terminated(MessageDefn::parse)?
        })
    }
}

pub fn define_messages(input: TokenStream) -> TokenStream
{
    let input = parse_macro_input!(input as MessageDefnList);

    generate_message_list(input)
}

fn generate_message_list(input: MessageDefnList) -> TokenStream
{
    let mut out = proc_macro2::TokenStream::new();

    for message in input.messages
    {
        let name = message.name;
        let typename = message.typename;
        let format_str = message.value;
        let aliases = message.aliases.iter();

        let mut message_args = Vec::new();
        let mut message_argtypes = Vec::new();

        let mut format_args = Vec::new();
        let mut format_values = Vec::new();

        let mut need_source = false;
        let mut need_target = false;

        for arg_or_targ in message.args
        {
            match arg_or_targ {
                MessageArg::Source(_) => {
                    need_source = true;
                }
                MessageArg::Target(_) => {
                    need_target = true;
                },
                MessageArg::Arg(arg) => {
                    message_args.push(arg.name.clone());
                    message_argtypes.push(arg.typename.clone());
                    format_args.push(arg.name.clone());
                    let fval_name = arg.name;
                    let fval_val = if let Some(e) = arg.expr {
                        quote!(#fval_name.#e)
                    } else {
                        quote!(#fval_name)
                    };
                    format_values.push(fval_val);
                }
            }
        }

        let (source_arg, source_def) = if need_source {
            (Some(quote!(source: &impl crate::ircd::irc::messages::MessageSource, )), Some(quote!(source = source.format(), )))
        } else {
            (None, None)
        };

        let (target_arg, target_def) = if need_target {
            (Some(quote!(target: &impl crate::ircd::irc::messages::MessageTarget, )), Some(quote!(target = target.format(), )))
        } else {
            (None, None)
        };

        out.extend(quote!(
            #[derive(Debug,Clone)]
            pub struct #typename(String);
            #( pub type #aliases = #typename; )*

            impl #typename
            {
                pub fn new(#source_arg #target_arg #( #message_args: #message_argtypes ),* ) -> Self
                {
                    Self(format!(concat!(#format_str, "\r\n"),
                                 #source_def
                                 #target_def
                                 #( #format_args = #format_values),*
                            ))
                }
            }
        ));

        if message.is_numeric
        {
            out.extend(quote!(
                impl crate::ircd::irc::messages::Numeric for #typename
                {
                    fn format_for(&self,
                                  source: &dyn crate::ircd::irc::messages::MessageSource,
                                  target: &dyn crate::ircd::irc::messages::MessageTarget
                                )
                            -> crate::ircd::irc::messages::TargetedNumeric
                    {
                        crate::ircd::irc::messages::TargetedNumeric(format!(":{source} {numeric} {target} {body}",
                                                                            source=source.format(),
                                                                            numeric=#name,
                                                                            target=target.format(),
                                                                            body=self.0
                                                                        ))
                    }
                }

                impl #typename
                {
                    pub fn new_for(source: &dyn crate::ircd::irc::messages::MessageSource,
                               target: &dyn crate::ircd::irc::messages::MessageTarget,
                               #( #message_args: #message_argtypes ),*
                            ) -> crate::ircd::irc::messages::TargetedNumeric
                    {
                        crate::ircd::irc::messages::TargetedNumeric(
                                     format!(concat!(":{source} {numeric} {target} ", #format_str, "\r\n"),
                                     source=source.format(),
                                     numeric=#name,
                                     target=target.format(),
                                     #( #format_args = #format_values),*
                                ))
                    }
                    }
            ));
        }
        else
        {
            out.extend(quote!(
                impl std::fmt::Display for #typename
                {
                    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result
                    {
                        self.0.fmt(f)
                    }
                }

                impl crate::ircd::irc::messages::Message for #typename
                { }
            ));
        }
    }

    //panic!("{}", out);

    out.into()
}