-- **

/* Remove stored function to search data from user tables by ID or nickname or email. */
DROP FUNCTION IF EXISTS find_user;

-- **

/* Drop trigger "trg_aft_ins_user_ins_session". */
DROP TRIGGER IF EXISTS trg_aft_ins_user_ins_session ON users;

/* Remove function "fn_aft_ins_user_ins_session". */
DROP FUNCTION IF EXISTS fn_aft_ins_user_ins_session;

-- **

/* Drop the "sessions" table. */
DROP TABLE IF EXISTS sessions;

-- **


/* Deleting the "users" table. */

DROP INDEX IF EXISTS uq_idx_users_email;
DROP INDEX IF EXISTS uq_idx_users_nickname;

DROP TABLE IF EXISTS users;

-- **

/* Remove type "user_role". */
DROP TYPE IF EXISTS user_role;

-- **
