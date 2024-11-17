use super::*;

#[derive(Queryable, Selectable, Identifiable, Associations, Insertable)]
#[diesel(table_name = crate::schema::messages)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(belongs_to(Channel, foreign_key = target_channel))]
#[diesel(belongs_to(HistoricUser, foreign_key = source_user))]
#[derive(Debug)]
pub struct Message {
    pub id: Uuid,
    pub source_user: i32,
    pub target_channel: i64,
    pub text: String,
    pub message_type: crate::types::MessageType,
    /// Timestamp of the *update* introducing the message.
    ///
    /// This is usually the same second as the one in [`id`] (a UUIDv7), but is
    /// occasionally 1 second later, because the message id is created before being
    /// pushed to the log.
    /// It can also before significantly different, because both are based on the
    /// system clock, which can change arbitrarily.
    pub timestamp: chrono::NaiveDateTime,
}
