/* Create a new "user_role" type. */
CREATE TYPE user_role AS ENUM ('admin', 'moderator', 'user');

/* Add column "role" to table "users". */
ALTER TABLE users ADD COLUMN "role" user_role NOT NULL DEFAULT 'user';
