-- Your SQL goes here

CREATE TABLE historic_users (
    id SERIAL PRIMARY KEY,
    user_id int8 NOT NULL,
    user_serial INTEGER NOT NULL,
    nick VARCHAR NOT NULL,
    ident VARCHAR NOT NULL,
    vhost VARCHAR NOT NULL,
    account_name VARCHAR,
    last_timestamp TIMESTAMP,

    UNIQUE (user_id, user_serial)
);

CREATE INDEX historic_users_by_user_id ON historic_users (user_id);

CREATE TABLE channels (
    id int8 PRIMARY KEY,
    name VARCHAR NOT NULL
);

CREATE TABLE messages (
    id uuid PRIMARY KEY,
    source_user INTEGER NOT NULL REFERENCES historic_users(id),
    target_channel int8 NOT NULL REFERENCES channels(id),
    text VARCHAR NOT NULL
);

