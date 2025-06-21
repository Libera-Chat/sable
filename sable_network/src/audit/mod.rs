//! Types providing audit log functionality.

use std::{fmt::Write, net::IpAddr};

use crate::{
    id::*,
    network::{event::*, state::*, wrapper::WrappedUser},
    node::NetworkNode,
};

pub struct AuditLogger<'a> {
    node: &'a NetworkNode,
    user: Option<UserId>,
    ip: IpAddr,
    action: String,
}

pub struct AuditLoggerEntry<'a> {
    node: &'a NetworkNode,
    pub category: AuditLogCategory,
    pub source_id: Option<UserId>,
    pub source_addr: Option<std::net::IpAddr>,
    pub source_str: String,
    pub action: String,
    pub target_id: Option<UserId>,
    pub target_str: Option<String>,
    pub target_duration: Option<i64>,
    pub reason: Option<String>,
}

fn format_source(node: &NetworkNode, id: Option<UserId>, ip: Option<IpAddr>) -> String {
    let mut ret = if let Some(id) = id {
        if let Ok(source_user) = node.network().user(id) {
            let mut source = source_user.nuh();
            if let Ok(Some(account)) = source_user.account() {
                write!(source, "[{}]", account.name()).expect("failed to write to string?");
            } else {
                write!(source, "[]").expect("failed to write to string?");
            }
            source
        } else {
            "<unknown user>".to_string()
        }
    } else {
        "<unregistered user>".to_string()
    };
    if let Some(ip) = ip {
        write!(ret, "{{{ip}}}").expect("failed to write to string?");
    }
    ret
}

impl<'a> AuditLogger<'a> {
    pub fn new(node: &'a NetworkNode, user: Option<UserId>, ip: IpAddr, action: String) -> Self {
        Self {
            node,
            user,
            ip,
            action,
        }
    }

    pub fn entry(&self, category: AuditLogCategory) -> AuditLoggerEntry<'a> {
        AuditLoggerEntry {
            node: self.node,
            category,
            source_id: self.user,
            source_addr: Some(self.ip),
            source_str: format_source(self.node, self.user, Some(self.ip)),
            action: self.action.clone(),
            target_id: None,
            target_str: None,
            target_duration: None,
            reason: None,
        }
    }

    pub fn general(&self) -> AuditLoggerEntry<'a> {
        self.entry(AuditLogCategory::General)
    }

    pub fn ban(&self) -> AuditLoggerEntry<'a> {
        self.entry(AuditLogCategory::NetworkBan)
    }

    pub fn kill(&self) -> AuditLoggerEntry<'a> {
        self.entry(AuditLogCategory::ServerKill)
    }
}

impl AuditLoggerEntry<'_> {
    pub fn source(mut self, id: Option<UserId>, ip: Option<IpAddr>) -> Self {
        self.source_id = id;
        self.source_str = format_source(self.node, id, ip);
        self
    }

    pub fn target_user(mut self, id: UserId) -> Self {
        self.target_id = Some(id);
        self.target_str = Some(format_source(self.node, Some(id), None));
        self
    }

    pub fn target_str(mut self, target: String) -> Self {
        self.target_str = Some(target);
        self
    }

    pub fn action(mut self, action: String) -> Self {
        self.action = action;
        self
    }

    pub fn target_duration(mut self, duration: i64) -> Self {
        self.target_duration = Some(duration);
        self
    }

    pub fn reason(mut self, reason: String) -> Self {
        self.reason = Some(reason);
        self
    }

    pub fn log(self) {
        let id = self.node.ids().next();
        let timestamp = crate::utils::now();

        tracing::info!(target: "audit",
          ?id,
          category = ?self.category,
          timestamp,
          source_id = ?self.source_id,
          source_addr = ?self.source_addr,
          source_str = self.source_str,
          action = self.action,
          target_id = ?self.target_id,
          target_str = self.target_str,
          target_duration = self.target_duration,
          reason = self.reason,
        );

        let entry = AuditLogEntry {
            id,
            category: self.category,
            timestamp,
            source_id: self.source_id,
            source_addr: self.source_addr,
            source_str: self.source_str,
            action: self.action,
            target_id: self.target_id,
            target_str: self.target_str,
            target_duration: self.target_duration,
            reason: self.reason,
        };
        self.node
            .submit_event(entry.id, details::NewAuditLogEntry { entry });
    }
}
