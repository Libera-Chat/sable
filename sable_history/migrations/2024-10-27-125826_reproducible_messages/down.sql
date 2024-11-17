DROP INDEX messages_by_timestamp;

ALTER TABLE
    DROP COLUMN message_type,
    DROP COLUMN timestamp;

DROP TYPE "MessageType";
