-- Removing entities: "blocked_users", "chat_messages", chat_message_logs.

-- **

-- Removing entities: "blocked_users".

/* Remove stored function to get the entity from "blocked_users". */
DROP FUNCTION IF EXISTS get_blocked_user;

/* Remove stored function that will filter "blocked_user" entities by the specified parameters. */
DROP FUNCTION IF EXISTS filter_blocked_users;

/* Remove stored function to delete the entity in "blocked_users". */
DROP FUNCTION IF EXISTS delete_blocked_user;

/* Remove stored function to add a new entry to "blocked_users". */
DROP FUNCTION IF EXISTS create_blocked_user;

-- **

/* Remove the indexes on the "blocked_users" table. */
DROP INDEX IF EXISTS uq_idx_blocked_users_blocked_id_user_id;

/* Remove the indexes on the "blocked_users" table. */
DROP INDEX IF EXISTS idx_blocked_users_user_id;

/* Remove the "blocked_users" table. */
DROP TABLE IF EXISTS blocked_users;

-- **

-- Removing entities: "chat_messages", chat_message_logs.

/* Remove stored function to get an array of entities in "chat_message_logs". */
DROP FUNCTION IF EXISTS get_chat_message_log;

-- **

/* Remove stored function to delete the entry in "chat_messages". */
DROP FUNCTION IF EXISTS delete_chat_message;

/* Remove stored function to modify the entry in "chat_messages". */
DROP FUNCTION IF EXISTS modify_chat_message;

/* Remove stored function to add a new entry to "chat_messages". */
DROP FUNCTION IF EXISTS create_chat_message;

/* Remove stored function that will filter "chat_message" entities. */
DROP FUNCTION IF EXISTS filter_chat_messages;

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
