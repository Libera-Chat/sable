use super::*;

use quote::{quote, quote_spanned};
use syn::{
    parenthesized, parse::Parse, parse_macro_input, token::In, Ident, ItemFn, LitStr, Token,
};

struct CommandHandlerAttr {
    command_name: LitStr,
    dispatcher: Option<LitStr>,
}

impl Parse for CommandHandlerAttr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let command_name = input.parse()?;
        let dispatcher = if input.parse::<Token![,]>().is_ok() {
            let content;
            input.parse::<In>()?;
            let _paren = parenthesized!(content in input);
            Some(content.parse()?)
        } else {
            None
        };
        Ok(Self {
            command_name,
            dispatcher,
        })
    }
}

pub fn command_handler(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(attr as CommandHandlerAttr);
    let item = parse_macro_input!(item as ItemFn);

    let name = &item.sig.ident;
    let asyncness = &item.sig.asyncness;

    let command_name = input.command_name;

    for c in command_name.value().chars() {
        if !c.is_ascii_uppercase() {
            return quote_spanned!(command_name.span()=> compile_error!("Command names should be uppercase")).into();
        }
    }

    let dispatcher = match input.dispatcher {
        Some(name) => quote!( Some( #name ) ),
        None => quote!(None),
    };

    let body = if asyncness.is_none() {
        quote!(
            if let Err(e) = crate::command::plumbing::call_handler(ctx.as_ref(), &super::#name, ctx.args())
            {
                ctx.notify_error(e);
            }
            None
        )
    } else {
        quote!(
            Some(Box::pin(async move {
                if let Err(e) = crate::command::plumbing::call_handler_async(ctx.as_ref(), &super::#name, ctx.args()).await
                {
                    ctx.notify_error(e);
                }
            }))
        )
    };

    let reg_mod_name = Ident::new(
        &format!(
            "register_{}_for_{}",
            name,
            command_name.value().to_ascii_lowercase()
        ),
        name.span(),
    );

    quote!(
        #item

        mod #reg_mod_name
        {
            use crate::command::Command;

            fn call_proxy<'a>(ctx: Box<dyn crate::command::Command + 'a>) -> Option<crate::command::AsyncHandler<'a>>
            {
                #body
            }

            inventory::submit!(crate::command::CommandRegistration {
                command: #command_name,
                dispatcher: #dispatcher,
                handler: call_proxy
            });
        }
    ).into()
}
