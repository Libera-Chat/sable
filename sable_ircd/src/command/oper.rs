use super::*;
use event::*;
use state::{
    AuditLogCategory,
    AuditLogField
};

command_handler!("OPER" => OperHandler {
    fn min_parameters(&self) -> usize { 2 }

    fn handle_user(&mut self, source: &wrapper::User, cmd: &ClientCommand) -> CommandResult
    {
        let net = self.server.network();
        let oper_name = &cmd.args[0];
        let password = &cmd.args[1];

        self.server.policy().user_can_oper(source)?;

        if let Some(conf) = self.find_oper_block(&*net, source, oper_name)
        {
            if self.server.policy().authenticate(conf, oper_name, password)
            {
                let audit = details::NewAuditLogEntry {
                    category: AuditLogCategory::General,
                    fields: vec![
                        (AuditLogField::Source, source.nuh()),
                        (AuditLogField::ActionType, "OPER".to_string()),
                    ]
                };
                self.action(CommandAction::state_change(self.server.ids().next_audit_log_entry(), audit))?;

                self.action(CommandAction::state_change(source.id(), details::OperUp {
                    oper_name: oper_name.clone()
                }))?;
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
});

impl OperHandler<'_>
{
    fn find_oper_block<'a>(&'a self, net: &'a Network, _user: &wrapper::User, oper_name: &str) -> Option<&'a config::OperConfig>
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
}