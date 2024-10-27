use super::*;

#[derive(Queryable, Selectable, Identifiable, Associations, Insertable)]
#[diesel(table_name = crate::schema::messages)]
#[diesel(check_for_backend(diesel::pg::Pg))]
#[diesel(belongs_to(Channel, foreign_key = target_channel))]
#[diesel(belongs_to(HistoricUser, foreign_key = source_user))]
pub struct Message {
    pub id: Uuid,
    pub source_user: i32,
    pub target_channel: i64,
    pub text: String,
    pub message_type: crate::types::MessageType,
}
