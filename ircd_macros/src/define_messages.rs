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
    syn::custom_keyword!(target);
}

enum MessageArg
{
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
    _arrow1: Token![=>],
    _brace: token::Brace,
      _paren: token::Paren,
        args: Punctuated<MessageArg, Token![,]>,
      _arrow2: Token![=>],
      value: LitStr,
}

struct MessageDefnList
{
    messages: Punctuated<MessageDefn, Token![,]>
}

struct NumericDefn(MessageDefn);
struct NumericDefnList(MessageDefnList);

impl Parse for MessageArg
{
    fn parse(input: ParseStream) -> Result<Self>
    {
        let lookahead = input.lookahead1();
        if lookahead.peek(kw::target)
        {
            Ok(Self::Target(input.parse()?))
        } else {
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

        let name: Ident = input.parse()?;

        Ok(Self {
            is_numeric: false,
            name: name.to_string(),
            typename: name,
            _arrow1: input.parse()?,
            _brace: braced!(content1 in input),
            _paren: parenthesized!(content2 in content1),
            args: content2.parse_terminated(MessageArg::parse)?,
            _arrow2: content1.parse()?,
            value: content1.parse()?
        })
    }
}

impl NumericDefn
{
    fn parse(input: ParseStream) -> Result<MessageDefn>
    {
        let content1;
        let content2;

        let number = input.parse::<LitInt>()?.to_string();
        let typename = Ident::new(&format!("Numeric{}", number), Span::call_site());

        Ok(MessageDefn {
            is_numeric: true,
            name: number,
            typename: typename,
            _arrow1: input.parse()?,
            _brace: braced!(content1 in input),
            _paren: parenthesized!(content2 in content1),
            args: content2.parse_terminated(MessageArg::parse)?,
            _arrow2: content1.parse()?,
            value: content1.parse()?
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

impl Parse for NumericDefnList
{
    fn parse(input: ParseStream) -> Result<Self>
    {
        Ok(Self(MessageDefnList {
            messages: input.parse_terminated(NumericDefn::parse)?
        }))
    }
}

pub fn define_messages(input: TokenStream) -> TokenStream
{
    let input = parse_macro_input!(input as MessageDefnList);

    generate_message_list(input)
}

pub fn define_numerics(input: TokenStream) -> TokenStream
{
    let input = parse_macro_input!(input as NumericDefnList);

    generate_message_list(input.0)
}

fn generate_message_list(input: MessageDefnList) -> TokenStream
{
    let mut out = proc_macro2::TokenStream::new();

    for message in input.messages
    {
        let name = message.name;
        let typename = message.typename;
        let format_str = message.value;

        let mut message_args = Vec::new();
        let mut message_argtypes = Vec::new();

        let mut format_args = Vec::new();
        let mut format_values = Vec::new();

        let mut need_target = message.is_numeric;

        for arg_or_targ in message.args
        {
            match arg_or_targ {
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

        let (target_arg, target_def) = if need_target {
            (Some(quote!(target: &impl MessageTarget, )), Some(quote!(target = target.format(), )))
        } else {
            (None, None)
        };

        let prefix = if message.is_numeric {
            Some(quote!(":{source} ", #name, " {target} ", ))
        } else {
            None
        };

        out.extend(quote!(
            pub struct #typename(String);

            impl #typename
            {
                pub fn new(source: &impl MessageSource, #target_arg #( #message_args: #message_argtypes ),* ) -> Self
                {
                    Self(format!(concat!(#prefix #format_str, "\r\n"), source = source.format(), #target_def #( #format_args = #format_values),* ))
                }
            }

            impl std::fmt::Display for #typename
            {
                fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result
                {
                    self.0.fmt(f)
                }
            }

            impl crate::ircd::irc::message::Message for #typename
            { }
        ));
    }

    //panic!("{}", out);

    out.into()
}