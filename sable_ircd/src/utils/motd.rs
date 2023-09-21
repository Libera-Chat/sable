use crate::errors::*;
use crate::messages::numeric;
use crate::messages::*;
use crate::*;
use sable_network::prelude::*;

pub fn send_motd(
    server: &ClientServer,
    to: impl MessageSink,
    to_user: &wrapper::User,
) -> HandleResult {
    match &server.info_strings.motd {
        None => to.send(numeric::NoMotd::new().format_for(server, to_user)),
        Some(motd) => {
            to.send(numeric::MotdStart::new(server.name()).format_for(server, to_user));
            for ele in motd {
                to.send(numeric::Motd::new(ele).format_for(server, to_user))
            }

            to.send(numeric::EndOfMotd::new().format_for(server, to_user));
        }
    }

    Ok(())
}
