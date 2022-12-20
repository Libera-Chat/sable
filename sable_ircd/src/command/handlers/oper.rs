use super::*;
use event::*;
use state::{
    AuditLogCategory,
    AuditLogField
};

#[command_handler("OPER")]
fn handle_oper(server: &ClientServer, net: &Network, source: UserSource,
               oper_name: &str, password: &str) -> CommandResult
{
    server.policy().user_can_oper(&source)?;

    if let Some(conf) = find_oper_block(net, &source, oper_name)
    {
        if server.policy().authenticate(conf, oper_name, password)
        {
            let audit = details::NewAuditLogEntry {
                category: AuditLogCategory::General,
                fields: vec![
                    (AuditLogField::Source, source.nuh()),
                    (AuditLogField::ActionType, "OPER".to_string()),
                ]
            };
            server.add_action(CommandAction::state_change(server.ids().next_audit_log_entry(), audit));

            server.add_action(CommandAction::state_change(source.id(), details::OperUp {
                oper_name: oper_name.to_owned()
            }));
            Ok(())
        }
        else
        {
            numeric_error!(NoOperConf)
        }
    }
    else
    {
        numeric_error!(NoOperConf)
    }
}


fn find_oper_block<'a>(net: &'a Network, _user: &wrapper::User, oper_name: &str) -> Option<&'a config::OperConfig>
{
    let conf = net.config();

    for block in &conf.opers
    {
        if block.name == oper_name
        {
            return Some(block)
        }
    }
    None
}
