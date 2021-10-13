use super::*;

use quote::quote;
use syn::{parse_macro_input, Result, Lit, Token, Ident};
use syn::parse::{Parse, ParseStream};

struct CommandHandlerDefn
{
    command: Lit,
    _comma: Token![,],
    name: Ident,
}

impl Parse for CommandHandlerDefn
{
    fn parse(input: ParseStream) -> Result<Self>
    {
        Ok(CommandHandlerDefn {
            command: input.parse()?,
            _comma: input.parse()?,
            name: input.parse()?,
        })
    }
}

pub fn command_handler(input: TokenStream) -> TokenStream
{
    let defn = parse_macro_input!(input as CommandHandlerDefn);
    let name = defn.name;
    let cmd = defn.command;

    quote!(
        pub struct #name;

        inventory::submit! {
            CommandRegistration {
                command: #cmd.to_string(),
                handler: Box::new(#name),
            }
        }
    ).into()
}