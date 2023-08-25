use super::*;
use event::*;

#[command_handler("OPER")]
fn handle_oper(
    server: &ClientServer,
    net: &Network,
    source: UserSource,
    audit: AuditLogger,
    oper_name: &str,
    password: &str,
) -> CommandResult {
    server.policy().user_can_oper(&source)?;

    if let Some(conf) = find_oper_block(net, &source, oper_name) {
        if server.policy().authenticate(conf, oper_name, password) {
            audit.general().log();

            server.add_action(CommandAction::state_change(
                source.id(),
                details::OperUp {
                    oper_name: oper_name.to_owned(),
                },
            ));
            Ok(())
        } else {
            numeric_error!(NoOperConf)
        }
    } else {
        numeric_error!(NoOperConf)
    }
}

fn find_oper_block<'a>(
    net: &'a Network,
    _user: &wrapper::User,
    oper_name: &str,
) -> Option<&'a config::OperConfig> {
    let conf = net.config();

    for block in &conf.opers {
        if block.name == oper_name {
            return Some(block);
        }
    }
    None
}
