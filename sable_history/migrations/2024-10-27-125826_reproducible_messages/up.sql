CREATE TYPE "Message_Type" AS ENUM ('privmsg', 'notice');

ALTER TABLE messages
    ADD COLUMN message_type "Message_Type" NOT NULL;
