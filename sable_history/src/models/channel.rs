use super::*;

#[derive(Queryable, Selectable, Identifiable, Insertable)]
#[diesel(table_name = crate::schema::channels)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Channel {
    pub id: i64,
    pub name: String,
}
