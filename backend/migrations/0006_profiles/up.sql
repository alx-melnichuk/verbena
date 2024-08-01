
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

-- call create_user2('nickname1', 'email1','passwd1','user');

/* Create a stored procedure to add a new user. */
CREATE OR REPLACE PROCEDURE create_user2(
  IN _nickname VARCHAR,
  IN _email VARCHAR,
  IN _password VARCHAR,
  IN _role user_role,
  INOUT "id" INTEGER DEFAULT NULL,
  INOUT nickname VARCHAR DEFAULT NULL,
  INOUT email VARCHAR DEFAULT NULL,
  INOUT "password" VARCHAR DEFAULT NULL,
  INOUT "role" VARCHAR DEFAULT NULL,
  INOUT created_at TIMESTAMP WITH TIME ZONE DEFAULT NULL,
  INOUT updated_at TIMESTAMP WITH TIME ZONE DEFAULT NULL
) LANGUAGE 'plpgsql'
AS $$
BEGIN
  INSERT INTO users (nickname, email, "password", "role")
  VALUES (_nickname, _email, _password, _role)
  RETURNING
      users.id, users.nickname, users.email, users."password", users."role", users.created_at, users.updated_at
    INTO 
      "id", nickname, email, "password", "role", created_at, updated_at;

  RETURN;
END ;
$$;
