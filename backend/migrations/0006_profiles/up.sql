
/* Create "profiles" table. */
CREATE TABLE profiles (
    user_id INT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    /* Link to user avatar, optional */
    avatar VARCHAR(255) NULL,
    /* user description */
    descript TEXT DEFAULT '' NOT NULL,
    /* Default color theme. 'light','dark' */
    theme VARCHAR(32) DEFAULT 'light' NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    PRIMARY KEY (user_id)
);

SELECT diesel_manage_updated_at('profiles');

/* Add data from the "users" table. */
INSERT INTO profiles (user_id, created_at, updated_at) 
(SELECT id, created_at, updated_at FROM users);

/* Stored function to retrieve data from the "profiles" and "users" tables. */
CREATE OR REPLACE FUNCTION get_profile_user(
  IN id1 INTEGER,
  OUT user_id INTEGER,
  OUT nickname VARCHAR,
  OUT email VARCHAR,
  OUT "role" user_role,
  OUT avatar VARCHAR,
  OUT descript TEXT,
  OUT theme VARCHAR,
  OUT created_at TIMESTAMP WITH TIME ZONE,
  OUT updated_at TIMESTAMP WITH TIME ZONE
) RETURNS SETOF record LANGUAGE sql
AS $$
  SELECT
    u.id AS user_id, 
    u.nickname,
    u.email,
    u."role",
    p.avatar,
    p.descript,
    p.theme,
    u.created_at,
    CASE WHEN u.updated_at > p.updated_at
      THEN u.updated_at ELSE p.updated_at END as updated_at
  FROM users u, profiles p
  WHERE u.id = id1 AND u.id = p.user_id;
$$;

/* Create a stored procedure to add a new user. */
CREATE OR REPLACE FUNCTION create_user6(
  IN _nickname VARCHAR,
  IN _email VARCHAR,
  IN _password VARCHAR,
  -- IN _role user_role,
  OUT user_id INTEGER,
  OUT nickname VARCHAR,
  OUT email VARCHAR,
  OUT "role" user_role,
  OUT avatar VARCHAR,
  OUT descript TEXT,
  OUT theme VARCHAR,
  OUT created_at TIMESTAMP WITH TIME ZONE,
  OUT updated_at TIMESTAMP WITH TIME ZONE
) RETURNS SETOF record LANGUAGE plpgsql
AS $$
DECLARE 
  rec1 RECORD;
  rec2 RECORD;
BEGIN
  -- Add a new entry to the "users" table.
  INSERT INTO users(nickname, email, "password")
  VALUES (_nickname, _email, _password)
  RETURNING users.id, users.nickname, users.email, users."password", users."role", users.created_at, users.updated_at
    INTO rec1;

  -- Add a new entry to the "profiles" table.
  INSERT INTO profiles(user_id)
  VALUES (rec1.id)
  RETURNING profiles.user_id, profiles.avatar, profiles.descript, profiles.theme, profiles.created_at, profiles.updated_at
    INTO rec2;

  -- Add a new entry to the "sessions" table.
  INSERT INTO sessions(user_id)
  VALUES (rec1.id);

  RETURN QUERY SELECT
    rec2.user_id, rec1.nickname, rec1.email, rec1."role", rec2.avatar, rec2.descript, rec2.theme,
    rec1.created_at,
    CASE WHEN rec1.updated_at > rec2.updated_at THEN rec1.updated_at ELSE rec2.updated_at END as updated_at;
END;
$$;
