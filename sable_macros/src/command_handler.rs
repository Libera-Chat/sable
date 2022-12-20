use super::*;

use quote::quote;
use syn::{
    parse_macro_input,
    LitStr, ItemFn, Ident,
};

pub fn command_handler(attr: TokenStream, item: TokenStream) -> TokenStream
{
    let command_name = parse_macro_input!(attr as LitStr);
    let item = parse_macro_input!(item as ItemFn);

    let name = &item.sig.ident;
    let asyncness = &item.sig.asyncness;

    let body = if asyncness.is_none() {
        quote!(
            if let Err(e) = crate::command::plumbing::call_handler(&ctx, &super::#name, &ctx.args)
            {
                ctx.notify_error(e);
            }
            None
        )
    } else {
        quote!(
            Some(Box::pin(async move {
                if let Err(e) = crate::command::plumbing::call_handler_async(&ctx, &super::#name, &ctx.args).await
                {
                    ctx.notify_error(e);
                }
            }))
        )
    };

    let reg_mod_name = Ident::new(&format!("register_{}", name), name.span());

    quote!(
        #item

        mod #reg_mod_name
        {
            use crate::command::CommandContext;

            fn call_proxy<'a>(ctx: crate::command::ClientCommand) -> Option<crate::command::AsyncHandler>
            {
                #body
            }

            inventory::submit!(crate::command::CommandRegistration {
                command: #command_name,
                handler: call_proxy
            });
        }
    ).into()
}