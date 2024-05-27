use super::*;

impl Network {
    pub(super) fn new_audit_log(
        &mut self,
        target: AuditLogEntryId,
        event: &Event,
        details: &details::NewAuditLogEntry,
        updates: &dyn NetworkUpdateReceiver,
    ) {
        self.audit_log.insert(target, details.entry.clone());

        let update = update::NewAuditLogEntry { entry: target };
        updates.notify(update, event);
    }
}
