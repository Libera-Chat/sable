CREATE TYPE "Message_Type" AS ENUM ('privmsg', 'notice');

ALTER TABLE messages
    ADD COLUMN message_type "Message_Type" NOT NULL,
    ADD COLUMN timestamp TIMESTAMP NOT NULL;

CREATE INDEX messages_by_timestamp ON messages USING BRIN (timestamp, id);
COMMENT ON INDEX messages_by_timestamp IS 'Includes the id in order to be a consistent total order across requests';
