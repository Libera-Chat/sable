use std::io::Write;

use diesel::pg::{Pg, PgValue};
use diesel::{deserialize, serialize};
use diesel::{AsExpression, FromSqlRow};

use crate::schema::sql_types::MessageType as SqlMessageType;

#[derive(Debug, PartialEq, FromSqlRow, AsExpression, Eq)]
#[diesel(sql_type = SqlMessageType)]
pub enum MessageType {
    Privmsg,
    Notice,
}

impl serialize::ToSql<SqlMessageType, Pg> for MessageType {
    fn to_sql<'b>(&'b self, out: &mut serialize::Output<'b, '_, Pg>) -> serialize::Result {
        match *self {
            MessageType::Privmsg => out.write_all(b"privmsg")?,
            MessageType::Notice => out.write_all(b"notice")?,
        }
        Ok(serialize::IsNull::No)
    }
}

impl deserialize::FromSql<SqlMessageType, Pg> for MessageType {
    fn from_sql(bytes: PgValue<'_>) -> deserialize::Result<Self> {
        match bytes.as_bytes() {
            b"privmsg" => Ok(MessageType::Privmsg),
            b"notice" => Ok(MessageType::Notice),
            _ => Err("Unrecognized enum variant for MessageType".into()),
        }
    }
}

impl From<sable_network::network::state::MessageType> for MessageType {
    fn from(value: sable_network::network::state::MessageType) -> Self {
        use sable_network::network::state::MessageType::*;
        match value {
            Privmsg => MessageType::Privmsg,
            Notice => MessageType::Notice,
        }
    }
}

impl From<MessageType> for sable_network::network::state::MessageType {
    fn from(value: MessageType) -> Self {
        use sable_network::network::state::MessageType::*;
        match value {
            MessageType::Privmsg => Privmsg,
            MessageType::Notice => Notice,
        }
    }
}
