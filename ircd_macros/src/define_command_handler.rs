use super::*;

use quote::quote;
use proc_macro2::Span;
use syn::{
    parse_macro_input,
    Result,
    Block,
    LitStr,
    Token,
    Ident,
};
use syn::parse::{Parse, ParseStream};

struct CommandHandlerDefn
{
    command: LitStr,
    _arrow: Token![=>],
    name: Ident,
    body: Block,
}

impl Parse for CommandHandlerDefn
{
    fn parse(input: ParseStream) -> Result<Self>
    {
        Ok(CommandHandlerDefn {
            command: input.parse()?,
            _arrow: input.parse()?,
            name: input.parse()?,
            body: input.parse()?,
        })
    }
}

pub fn command_handler(input: TokenStream) -> TokenStream
{
    let defn = parse_macro_input!(input as CommandHandlerDefn);
    let name = defn.name;
    let factory = Ident::new(&format!("{}Factory", name), Span::call_site());
    let cmd = defn.command;
    let body = defn.body;

    quote!(
        pub struct #name<'a>
        {
            server: &'a Server,
            processor: &'a CommandProcessor<'a>,
            actions: Vec<CommandAction>,
        }

        impl<'a> #name<'a>
        {
            pub fn new(server: &'a Server, proc: &'a CommandProcessor<'a>) -> Self
            {
                Self{ server: server, processor: proc, actions: Vec::new() }
            }

            pub fn action(&mut self, act: CommandAction) -> network::ValidationResult
            {
                if let CommandAction::StateChange(i, d) = &act {
                    self.server.network().validate(*i, d)?;
                }
                self.actions.push(act);
                Ok(())
            }
        }

        impl<'a> CommandHandler for #name<'a>
        #body

        impl IntoActions for #name<'_>
        {
            fn into_actions(&mut self) -> Vec<CommandAction>
            { std::mem::take(&mut self.actions) }
        }

        struct #factory;

        impl CommandHandlerFactory for #factory
        {
            fn create<'a>(&self, server: &'a Server, proc: &'a CommandProcessor<'a>) -> Box<dyn CommandHandler + 'a>
            { Box::new(#name::new(server, proc)) }
        }

        inventory::submit! {
            CommandRegistration {
                command: #cmd.to_string(),
                handler: Box::new(#factory),
            }
        }
    ).into()
}