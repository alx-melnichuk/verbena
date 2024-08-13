
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
CREATE OR REPLACE FUNCTION find_profile_user(
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

/* Create a stored function to add a new user, their profile, and their session. */
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

/* Create a stored function to modify a user and their profile. */
CREATE OR REPLACE FUNCTION modify_profile_user(
  INOUT user_id INTEGER,
  INOUT nickname VARCHAR,
  INOUT email VARCHAR,
  INOUT "password" VARCHAR,
  INOUT "role" user_role,
  INOUT avatar VARCHAR,
  INOUT descript TEXT,
  INOUT theme VARCHAR,
  OUT created_at TIMESTAMP WITH TIME ZONE,
  OUT updated_at TIMESTAMP WITH TIME ZONE
) RETURNS SETOF record LANGUAGE plpgsql
AS $$
DECLARE 
  sql1 TEXT;
  fields VARCHAR[];
  rec1 RECORD;
  rec2 RECORD;
BEGIN
  IF user_id IS NULL THEN
    RETURN;
  END IF;

  SELECT
    CAST(NULL AS INTEGER) AS user_id, CAST(NULL AS VARCHAR) AS nickname  , 
    CAST(NULL AS VARCHAR) AS email  , CAST(NULL AS VARCHAR) AS "password", CAST(NULL AS user_role) AS "role",
    CAST(NULL AS VARCHAR) AS avatar , CAST(NULL AS TEXT) AS descript     , CAST(NULL AS VARCHAR) AS theme,
    CAST(NULL AS TIMESTAMP WITH TIME ZONE) AS created_at, CAST(NULL AS TIMESTAMP WITH TIME ZONE) AS updated_at
  INTO rec1;

  SELECT
    CAST(NULL AS INTEGER) AS user_id, CAST(NULL AS VARCHAR) AS nickname  , 
    CAST(NULL AS VARCHAR) AS email  , CAST(NULL AS VARCHAR) AS "password", CAST(NULL AS user_role) AS "role",
    CAST(NULL AS VARCHAR) AS avatar , CAST(NULL AS TEXT) AS descript     , CAST(NULL AS VARCHAR) AS theme,
    CAST(NULL AS TIMESTAMP WITH TIME ZONE) AS created_at, CAST(NULL AS TIMESTAMP WITH TIME ZONE) AS updated_at
  INTO rec2;

  sql1 := '';
  fields := ARRAY[]::VARCHAR[];
  IF nickname IS NOT NULL AND LENGTH(nickname) > 0 THEN
    fields := ARRAY_APPEND(fields, 'nickname = ''' || nickname || '''');
  END IF;
  IF email IS NOT NULL AND LENGTH(email) > 0 THEN
    fields := ARRAY_APPEND(fields, 'email = ''' || email || '''');
  END IF;
  IF "password" IS NOT NULL AND LENGTH("password") > 0 THEN
    fields := ARRAY_APPEND(fields, '"password" = ''' || "password" || '''');
  END IF;
  IF "role" IS NOT NULL AND LENGTH("role"::VARCHAR) > 0 THEN
    fields := ARRAY_APPEND(fields, '"role" = ''' || "role" || '''');
  END IF;

  IF ARRAY_LENGTH(fields, 1) > 0 THEN
    sql1 := 'UPDATE users SET '
      || ARRAY_TO_STRING(fields, ',')
      || ' FROM profiles'  
      || ' WHERE users.id = profiles.user_id AND id = ' || user_id
      || ' RETURNING '
      || ' users.id AS user_id, users.nickname, users.email, users."password", users."role",'
      || ' profiles.avatar, profiles.descript, profiles.theme,'
      || ' users.created_at, users.updated_at'; 
    EXECUTE sql1 INTO rec1;
  END IF;

  sql1 := '';
  fields := ARRAY[]::VARCHAR[];
  IF avatar IS NOT NULL AND LENGTH(avatar) > 0 THEN
    fields := ARRAY_APPEND(fields, 'avatar = ''' || avatar || '''');
  END IF;
  IF descript IS NOT NULL AND LENGTH(descript) > 0 THEN
    fields := ARRAY_APPEND(fields, 'descript = ''' || descript || '''');
  END IF;
  IF theme IS NOT NULL AND LENGTH(theme) > 0 THEN
    fields := ARRAY_APPEND(fields, 'theme = ''' || theme || '''');
  END IF;

  IF ARRAY_LENGTH(fields, 1) > 0 THEN
    sql1 := 'UPDATE profiles SET '
      || ARRAY_TO_STRING(fields, ',')
      || ' FROM users'  
      || ' WHERE users.id = profiles.user_id AND profiles.user_id = ' || user_id
      || ' RETURNING '
      || ' users.id AS user_id, users.nickname, users.email, users."password", users."role",'
      || ' profiles.avatar, profiles.descript, profiles.theme,'
      || ' users.created_at, profiles.updated_at'; 
    EXECUTE sql1 INTO rec2;
  END IF;

  IF rec1.user_id IS NULL AND rec2.user_id IS NULL THEN
    RETURN;
  END IF;

  RETURN QUERY SELECT
    COALESCE(rec1.user_id   , rec2.user_id   ) AS user_id,
    COALESCE(rec1.nickname  , rec2.nickname  ) AS nickname,
    COALESCE(rec1.email     , rec2.email     ) AS email,
    COALESCE(rec1."password", rec2."password") AS "password",
    COALESCE(rec1."role"    , rec2."role"    ) AS "role",
    COALESCE(rec2.avatar    , rec1.avatar    ) AS avatar,
    COALESCE(rec2.descript  , rec1.descript  ) AS descript,
    COALESCE(rec2.theme     , rec1.theme     ) AS theme,
    COALESCE(rec1.created_at, rec2.created_at) AS created_at,
    COALESCE(rec2.updated_at, rec1.updated_at) AS updated_at;
END;
$$;

/* Create a stored function to delete a user, their profile, and their session. */
CREATE OR REPLACE FUNCTION delete_profile_user(
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
