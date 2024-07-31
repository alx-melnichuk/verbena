
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
  OUT "role" VARCHAR,
  OUT avatar VARCHAR,
  OUT descript VARCHAR,
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
