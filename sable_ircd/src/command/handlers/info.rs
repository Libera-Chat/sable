use super::*;

#[command_handler("INFO")]
/// Syntax: INFO
fn info_handler(response: &dyn CommandResponse, server: &ClientServer) -> CommandResult {
    let node = server.node();
    let version = node.version();
    let commit_date = node.commit_date();

    for line in &server.info_strings.info {
        response.numeric(make_numeric!(Info, line));
    }
    response.numeric(make_numeric!(Info, &format!("Version: {version}")));
    response.numeric(make_numeric!(Info, &format!("Commit date: {commit_date}")));
    // TODO: send configuration info to opers
    // see: https://github.com/solanum-ircd/solanum/blob/main/modules/m_info.c#L788
    response.numeric(make_numeric!(EndOfInfo));
    Ok(())
}
