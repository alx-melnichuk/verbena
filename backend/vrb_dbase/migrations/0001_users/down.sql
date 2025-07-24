/* Deleting the "users" table. */

DROP INDEX IF EXISTS uq_idx_users_nickname;
DROP INDEX IF EXISTS uq_idx_users_email;

DROP TABLE IF EXISTS users;
