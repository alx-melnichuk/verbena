/* Remove stored function to delete a user, their profile, and their session. */
-- DROP FUNCTION IF EXISTS delete_profile_user;

/* Remove stored function to modify a user and their profile. */
-- DROP FUNCTION IF EXISTS modify_profile_user;

/* Remove stored function to add a new user. */
-- DROP FUNCTION IF EXISTS create_profile_user;

/* Drop stored function for retrieving data from the "profiles", "password" and "users" tables by id or nickname, email. */
-- DROP FUNCTION IF EXISTS find_profile_user;


-- **

/* Remove stored function to add a new entry to "chat_messages".*/
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
