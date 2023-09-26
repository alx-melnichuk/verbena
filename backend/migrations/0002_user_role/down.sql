/* Remove the "role" column from the "users" table. */
ALTER TABLE users DROP COLUMN "role";

/* Remove type "user_role". */
DROP TYPE user_role;