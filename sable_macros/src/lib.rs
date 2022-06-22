#![allow(clippy::large_enum_variant)]
#![allow(clippy::eval_order_dependence)]

extern crate proc_macro;

use proc_macro::TokenStream;

mod define_event_details;

#[proc_macro]
pub fn event_details(input: TokenStream) -> TokenStream
{
    define_event_details::event_details(input)
}

#[proc_macro_attribute]
pub fn target_type(attr: TokenStream, item: TokenStream) -> TokenStream
{
    define_event_details::target_type_attribute(attr, item)
}

mod dispatch_event;

#[proc_macro]
pub fn dispatch_event(input: TokenStream) -> TokenStream
{
    dispatch_event::dispatch_event(input, false)
}

#[proc_macro]
pub fn dispatch_event_async(input: TokenStream) -> TokenStream
{
    dispatch_event::dispatch_event(input, true)
}

mod define_command_handler;

#[proc_macro]
pub fn command_handler(input: TokenStream) -> TokenStream
{
    define_command_handler::command_handler(input)
}

mod define_object_id;

#[proc_macro]
pub fn object_ids(input: TokenStream) -> TokenStream
{
    define_object_id::object_ids(input)
}

mod define_messages;

#[proc_macro]
pub fn define_messages(input: TokenStream) -> TokenStream
{
    define_messages::define_messages(input)
}

mod define_validated;

#[proc_macro]
pub fn define_validated(input: TokenStream) -> TokenStream
{
    define_validated::define_validated(input)
}

mod modeflags;

#[proc_macro]
pub fn mode_flags(input: TokenStream) -> TokenStream
{
    modeflags::mode_flags(input)
}
