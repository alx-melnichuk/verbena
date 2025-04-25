-- Removing entities: "chat_messages", chat_message_logs.

-- **

/* Remove stored function to get an array of entities in "chat_message_logs". */
DROP FUNCTION IF EXISTS get_chat_message_log;

-- **

/* Remove stored function to delete the entry in "chat_messages". */
DROP FUNCTION IF EXISTS delete_chat_message;

/* Remove stored function to modify the entry in "chat_messages". */
DROP FUNCTION IF EXISTS modify_chat_message;

/* Remove stored function to add a new entry to "chat_messages". */
DROP FUNCTION IF EXISTS create_chat_message;

-- **

/* Remove the indexes on the "chat_message_logs" table. */
DROP INDEX IF EXISTS idx_chat_message_logs_chat_message_id;

/* Remove the "chat_message_logs" table. */
DROP TABLE IF EXISTS chat_message_logs;

-- **

/* Remove the indexes on the "chat_messages" table. */
DROP INDEX IF EXISTS idx_chat_messages_user_id;
DROP INDEX IF EXISTS idx_chat_messages_stream_id;

/* Remove the "chat_messages" table. */
DROP TABLE IF EXISTS chat_messages;

-- **
