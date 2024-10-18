use crate::schema::historic_users;

use diesel::prelude::*;

#[derive(Queryable, Selectable, Identifiable)]
#[diesel(table_name = crate::schema::historic_users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct HistoricUser {
    pub id: i32,
    pub user_id: i64,
    pub user_serial: i32,
    pub nick: String,
    pub ident: String,
    pub vhost: String,
    pub account_name: Option<String>,
    pub last_timestamp: Option<chrono::NaiveDateTime>,
}

impl HistoricUser {
    #[diesel::dsl::auto_type(no_type_alias)]
    pub fn with_network_id<'a>(id: &'a sable_network::id::HistoricUserId) -> _ {
        use crate::schema::historic_users;
        let user_id: i64 = id.user().as_u64() as i64;
        let user_serial: i32 = id.serial() as i32;

        historic_users::user_id
            .eq(user_id)
            .and(historic_users::user_serial.eq(user_serial))
    }
}

#[derive(Insertable)]
#[diesel(table_name = crate::schema::historic_users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct NewHistoricUser {
    pub user_id: i64,
    pub user_serial: i32,
    pub nick: String,
    pub ident: String,
    pub vhost: String,
    pub account_name: Option<String>,
    pub last_timestamp: Option<chrono::NaiveDateTime>,
}

impl From<&sable_network::network::state::HistoricUser> for NewHistoricUser {
    fn from(value: &sable_network::network::state::HistoricUser) -> Self {
        // Deprecated warning comes from using NaiveDateTime::from_timestamp
        #[expect(deprecated)]
        Self {
            user_id: value.id.as_u64() as i64,
            user_serial: value.serial as i32,
            nick: value.nickname.to_string(),
            ident: value.user.to_string(),
            vhost: value.visible_host.to_string(),
            account_name: value.account.as_ref().map(ToString::to_string),
            last_timestamp: value
                .timestamp
                .map(|ts| chrono::NaiveDateTime::from_timestamp(ts, 0)),
        }
    }
}
