use super::*;

#[command_handler("ADMIN")]
fn handle_admin(server: &ClientServer, response: &dyn CommandResponse) -> CommandResult {
    response.numeric(make_numeric!(AdminMe));
    if let Some(admin_info) = &server.info_strings.admin_info {
        admin_info
            .server_location
            .as_ref()
            .map(|i| response.numeric(make_numeric!(AdminLocation1, i)));

        admin_info
            .description
            .as_ref()
            .map(|i| response.numeric(make_numeric!(AdminLocation2, i)));

        admin_info
            .admin_email
            .as_ref()
            .map(|i| response.numeric(make_numeric!(AdminEmail, i)));
    }
    Ok(())
}
