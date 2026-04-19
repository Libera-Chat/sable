use super::*;

#[command_handler("ADMIN")]
/// ADMIN
///
/// ADMIN shows the information that was set by the
/// administrator of the server. This information
/// can take any form that will fit in three lines
/// of text but is usually a list of contacts for
/// the persons that run the server.
fn handle_admin(server: &ClientServer, response: &dyn CommandResponse) -> CommandResult {
    response.numeric(make_numeric!(AdminMe, server.name()));
    if let Some(admin_info) = &server.info_strings.admin_info {
        if let Some(i) = admin_info.server_location.as_ref() {
            response.numeric(make_numeric!(AdminLocation1, i))
        }

        if let Some(i) = admin_info.description.as_ref() {
            response.numeric(make_numeric!(AdminLocation2, i))
        }

        if let Some(i) = admin_info.email.as_ref() {
            response.numeric(make_numeric!(AdminEmail, i))
        }
    }
    Ok(())
}
