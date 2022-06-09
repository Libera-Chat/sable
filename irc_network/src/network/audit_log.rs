use super::*;
use crate::update::*;

impl Network
{
    pub(super) fn new_audit_log(&mut self, target: AuditLogEntryId, event: &Event, details: &details::NewAuditLogEntry, updates: &dyn NetworkUpdateReceiver)
    {
        let entry = state::AuditLogEntry {
            id: target,
            timestamp: event.timestamp,
            category: details.category,
            fields: details.fields.clone(),
        };

        self.audit_log.insert(target, entry.clone());

        let update = update::NewAuditLogEntry { entry: entry };
        updates.notify(update);
    }
}