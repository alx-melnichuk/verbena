
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

/* Stored function for retrieving data from the "profiles", "password" and "users" tables by nickname or email. */
CREATE OR REPLACE FUNCTION find_profile_user_by_id_or_nickname_email(
  IN _id INTEGER,
  IN _nickname VARCHAR,
  IN _email VARCHAR,
  IN _is_password BOOLEAN, 
  OUT user_id INTEGER,
  OUT nickname VARCHAR,
  OUT email VARCHAR,
  OUT "password" VARCHAR,
  OUT "role" user_role,
  OUT avatar VARCHAR,
  OUT descript TEXT,
  OUT theme VARCHAR,
  OUT created_at TIMESTAMP WITH TIME ZONE,
  OUT updated_at TIMESTAMP WITH TIME ZONE
) RETURNS SETOF record LANGUAGE plpgsql
AS $$
DECLARE 
  sql_text TEXT;
BEGIN
  sql_text := 
   'SELECT
      "users".id AS user_id, 
      "users".nickname,
      "users".email,
    ' || 
    CASE WHEN _is_password THEN '"users"."password",'
       ELSE ' ''''::VARCHAR AS "password",'
    END ||
    ' "users"."role",
      "profiles".avatar,
      "profiles".descript,
      "profiles".theme,
      "users".created_at,
      CASE WHEN "users".updated_at > "profiles".updated_at
        THEN "users".updated_at ELSE "profiles".updated_at END as updated_at
    FROM 
      ("users" INNER JOIN "profiles" ON ("profiles"."user_id" = "users"."id"))
    ';

  IF _id IS NOT NULL THEN
    -- Add search condition by ID.
    sql_text := sql_text || ' WHERE ("users".id = ' || _id || ')';
  ELSE
    IF LENGTH(_nickname)=0 AND LENGTH(_email)=0 THEN
      RETURN;
    END IF;
    -- Add a search condition by nickname or email.
    sql_text := sql_text || ' WHERE ("users".nickname = ''' || _nickname || ''' OR "users".email = ''' || _email || ''')';
  END IF;

  RETURN QUERY EXECUTE sql_text || ' LIMIT 1';
END;
$$;

/* Create a stored procedure to add a new user, their profile, and their session. */
CREATE OR REPLACE FUNCTION create_profile_user(
  IN _nickname VARCHAR,
  IN _email VARCHAR,
  IN _password VARCHAR,
  IN _role user_role,
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
  INSERT INTO users(nickname, email, "password", "role")
  VALUES (_nickname, _email, _password, _role)
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

/* Create a stored procedure to delete a user, their profile, and their session. */
CREATE OR REPLACE FUNCTION delete_profile_user_by_user_id(
  IN _id INTEGER,
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
BEGIN
  -- delete the user, his profile and session.
  DELETE FROM "users"
  USING "profiles"
  WHERE 
   ("profiles"."user_id" = "users"."id") AND ("users"."id" = _id)
  RETURNING 
    "users".id AS user_id, 
    "users".nickname,
    "users".email,
    "users"."role",
    "profiles".avatar,
    "profiles".descript,
    "profiles".theme,
    "users".created_at,
    CASE WHEN "users".updated_at > "profiles".updated_at
    THEN "users".updated_at ELSE "profiles".updated_at END as updated_at
  INTO rec1;

  IF rec1.user_id IS NULL THEN
    RETURN;
  END IF;

  RETURN QUERY SELECT
    rec1.user_id, rec1.nickname, rec1.email, rec1."role", rec1.avatar, rec1.descript, rec1.theme,
    rec1.created_at, rec1.updated_at;
END;
$$;
