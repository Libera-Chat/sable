use sable_network::rpc::{RemoteServerRequestType, RemoteServerResponse};

use super::*;

pub struct ServicesTarget<'a> {
    name: ServerName,
    server: &'a ClientServer,
}

impl From<ServicesTarget<'_>> for ServerName {
    fn from(val: ServicesTarget<'_>) -> Self {
        val.name
    }
}

impl<'a> AmbientArgument<'a> for ServicesTarget<'a> {
    fn load_from(ctx: &'a dyn Command) -> Result<Self, CommandError> {
        Ok(Self {
            name: ctx
                .network()
                .current_services_server_name()
                .ok_or(CommandError::ServicesNotAvailable)?,
            server: ctx.server(),
        })
    }
}

impl ServicesTarget<'_> {
    pub async fn send_remote_request(
        &self,
        req: RemoteServerRequestType,
    ) -> Result<RemoteServerResponse, NetworkError> {
        self.server
            .node()
            .sync_log()
            .send_remote_request(self.name, req)
            .await
    }
}
