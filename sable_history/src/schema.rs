// @generated automatically by Diesel CLI.

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
    messages (id) {
        id -> Uuid,
        source_user -> Int4,
        target_channel -> Int8,
        text -> Varchar,
    }
}

diesel::joinable!(messages -> channels (target_channel));
diesel::joinable!(messages -> historic_users (source_user));

diesel::allow_tables_to_appear_in_same_query!(channels, historic_users, messages,);
