-- **

/* Create a new "user_role" type. */
CREATE TYPE user_role AS ENUM ('admin', 'moderator', 'user');

-- **

/* Creation of the "users" table. */

CREATE TABLE users (
    id SERIAL PRIMARY KEY NOT NULL,
    nickname VARCHAR(255) NOT NULL,
    email VARCHAR(255) NOT NULL,
    "password" VARCHAR(255) NOT NULL,
    "role" user_role DEFAULT 'user'::user_role NOT NULL,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

SELECT diesel_manage_updated_at('users');

CREATE UNIQUE INDEX uq_idx_users_nickname ON users(nickname);
CREATE UNIQUE INDEX uq_idx_users_email ON users(email);

-- **

/* Creation of the "sessions" table. */
CREATE TABLE sessions (
   user_id INT REFERENCES users(id) ON DELETE CASCADE,
   num_token INT,
   PRIMARY KEY (user_id)
);

-- **

/* Create a function to add a record to the "sessions" table
 after adding a record to the "users" table. */
CREATE OR REPLACE FUNCTION fn_aft_ins_user_ins_session() RETURNS TRIGGER AS $$
BEGIN
  -- Add a record to the sessions table for the new user.
  INSERT INTO sessions(user_id)
  VALUES(new.id);

  RETURN new;
END;
$$ LANGUAGE plpgsql;

/* Create a trigger after adding a record to the "users" tab.
(Automatically add records to the "sessions" tab.) */
CREATE TRIGGER trg_aft_ins_user_ins_session
  AFTER INSERT ON users FOR EACH ROW EXECUTE PROCEDURE fn_aft_ins_user_ins_session();

-- **

/* Stored function for retrieving data from the "users" tables by ID or nickname or email. */
CREATE OR REPLACE FUNCTION find_user(
  IN _id INTEGER,
  IN _nickname VARCHAR,
  IN _email VARCHAR,
  IN _is_password BOOLEAN,
  OUT id INTEGER,
  OUT nickname VARCHAR,
  OUT email VARCHAR,
  OUT "password" VARCHAR,
  OUT "role" user_role,
  OUT created_at TIMESTAMPTZ,
  OUT updated_at TIMESTAMPTZ
) RETURNS SETOF record LANGUAGE plpgsql
AS $$
DECLARE
  rec1 RECORD;
BEGIN
  IF _id IS NOT NULL THEN
    SELECT u.id, u.nickname, u.email, u."password", u."role", u.created_at, u.updated_at
    FROM "users" u
    WHERE u.id = _id
    INTO rec1;
  END IF;

  IF rec1 IS NULL AND LENGTH(coalesce(_nickname, '')) > 0 THEN
    SELECT u.id, u.nickname, u.email, u."password", u."role", u.created_at, u.updated_at
    FROM users u
    WHERE u.nickname = _nickname
    LIMIT 1
    INTO rec1;
  END IF;

  IF rec1 IS NULL AND LENGTH(coalesce(_email, '')) > 0 THEN
    SELECT u.id, u.nickname, u.email, u."password", u."role", u.created_at, u.updated_at
    FROM "users" u
    WHERE u.email = _email
    LIMIT 1
    INTO rec1;
  END IF;

  IF rec1 IS NOT NULL AND rec1.id IS NOT NULL THEN
    RETURN QUERY
      SELECT
        rec1.id,
        rec1.nickname,
        rec1.email,
        CASE WHEN _is_password THEN rec1."password" ELSE ''::VARCHAR END AS "password",
        rec1."role",
        rec1.created_at,
        rec1.updated_at;
  END IF;
END;
$$;

-- **
