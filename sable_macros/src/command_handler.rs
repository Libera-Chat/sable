use super::*;

use quote::{quote, quote_spanned};
use syn::{
    parenthesized, parse::Parse, parse_macro_input, token::In, Attribute, Ident, ItemFn, LitStr,
    Meta, MetaNameValue, Token,
};

struct CommandHandlerAttr {
    command_name: LitStr,
    aliases: Vec<LitStr>,
    dispatcher: Option<LitStr>,
    restricted: bool,
}

impl Parse for CommandHandlerAttr {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let command_name = input.parse()?;
        let mut aliases = vec![];
        let mut dispatcher = None;
        let mut restricted = false;
        while input.peek(Token![,]) {
            if !input.peek2(LitStr) {
                break;
            }
            let _ = input.parse::<Token![,]>();
            aliases.push(input.parse()?);
        }
        while input.peek(Token![,]) {
            let _ = input.parse::<Token![,]>()?;
            if input.peek(In) {
                let content;
                input.parse::<In>()?;
                let _paren = parenthesized!(content in input);
                dispatcher = Some(content.parse()?);
            } else if input.peek(Ident) {
                if input.parse::<Ident>()? == "restricted" {
                    restricted = true;
                }
            }
        }
        Ok(Self {
            command_name,
            aliases,
            dispatcher,
            restricted,
        })
    }
}

pub fn command_docs(attrs: &[Attribute]) -> Vec<String> {
    attrs
        .iter()
        .filter(|a| a.path.is_ident("doc"))
        .filter_map(|a| match a.parse_meta() {
            Ok(Meta::NameValue(MetaNameValue {
                lit: syn::Lit::Str(s),
                ..
            })) => Some(s.value()),
            _ => None,
        })
        .map(|s| s.strip_prefix(' ').unwrap_or(&s).trim_end().to_owned())
        .collect()
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

    let aliases = input.aliases;
    for alias in &aliases {
        for c in alias.value().chars() {
            if !c.is_ascii_uppercase() {
                return quote_spanned!(command_name.span()=> compile_error!("Command aliases should be uppercase")).into();
            }
        }
    }

    let dispatcher = match input.dispatcher {
        Some(name) => quote!( Some( #name ) ),
        None => quote!(None),
    };

    let restricted = input.restricted;

    let docs = command_docs(&item.attrs);

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
                aliases: &[ #(#aliases),* ],
                dispatcher: #dispatcher,
                handler: call_proxy,
                restricted: #restricted,
                docs: &[ #(#docs),* ],
            });
        }
    ).into()
}
