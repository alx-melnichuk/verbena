/* Remove the "role" column from the "users" table. */
ALTER TABLE users DROP COLUMN "role";

/* Remove type "user_role". */
DROP TYPE IF EXISTS user_role;

/* Drop the session table. */
DROP TABLE IF EXISTS sessions;