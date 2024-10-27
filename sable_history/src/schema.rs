// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::query_builder::QueryId, Clone, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "Message_Type"))]
    pub struct MessageType;
}

diesel::table! {
    channels (id) {
        id -> Int8,
        name -> Varchar,
    }
}

diesel::table! {
    historic_users (id) {
        id -> Int4,
        user_id -> Int8,
        user_serial -> Int4,
        nick -> Varchar,
        ident -> Varchar,
        vhost -> Varchar,
        account_name -> Nullable<Varchar>,
        last_timestamp -> Nullable<Timestamp>,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::MessageType;

    messages (id) {
        id -> Uuid,
        source_user -> Int4,
        target_channel -> Int8,
        text -> Varchar,
        message_type -> MessageType,
        timestamp -> Timestamp,
    }
}

diesel::joinable!(messages -> channels (target_channel));
diesel::joinable!(messages -> historic_users (source_user));

diesel::allow_tables_to_appear_in_same_query!(channels, historic_users, messages,);
