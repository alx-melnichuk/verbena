
/* Create "profiles" table. */
CREATE TABLE profiles (
    user_id INT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    /* Link to user avatar, optional */
    avatar VARCHAR(255) NULL,
    /* User description. */
    descript TEXT NULL,
    /* Default color theme. ('light','dark') */
    theme VARCHAR(32) NULL,
    /* Default locale. */
    locale VARCHAR(32) NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    PRIMARY KEY (user_id)
);

SELECT diesel_manage_updated_at('profiles');

-- **

/* Create a function to add a record to the "profiles" table
   after adding a record to the "users" table. */
CREATE OR REPLACE FUNCTION fn_aft_ins_user_ins_profile() RETURNS TRIGGER AS $$
BEGIN
  -- Add a record to the profiles table for the new user.
  INSERT INTO profiles(user_id)
  VALUES(new.id);

  RETURN new;
END;
$$ LANGUAGE plpgsql;

/* Create a trigger after adding a record to the "users" tab.
   (Automatically add records to the "profiles" tab.) */
CREATE TRIGGER trg_aft_ins_user_ins_profile
  AFTER INSERT ON users FOR EACH ROW EXECUTE PROCEDURE fn_aft_ins_user_ins_profile();

-- **

/* Add data from the "users" table. */
INSERT INTO profiles (user_id, created_at, updated_at) 
(SELECT id, created_at, updated_at FROM users);

-- **

/* Create a stored function for retrieving data from the "users" and "profiles" tables by ID. */
CREATE OR REPLACE FUNCTION get_user_profile_by_id(
  IN _user_id INTEGER,
  OUT user_id INTEGER,
  OUT nickname VARCHAR,
  OUT email VARCHAR,
  OUT "role" user_role,
  OUT avatar VARCHAR,
  OUT descript TEXT,
  OUT theme VARCHAR,
  OUT locale VARCHAR,
  OUT created_at TIMESTAMPTZ,
  OUT updated_at TIMESTAMPTZ
) RETURNS SETOF record LANGUAGE plpgsql
AS $$
DECLARE
  rec_user RECORD = NULL;
  rec_profile RECORD = NULL;
BEGIN
  IF _user_id IS NULL THEN
    RETURN;
  END IF;

  SELECT u.id, u.nickname, u.email, u."role", u.created_at, u.updated_at
  FROM "users" u
  WHERE u.id = _user_id
  INTO rec_user;

  SELECT p.user_id, p.avatar, p.descript, p.theme, p.locale, p.updated_at
  FROM profiles p
  WHERE p.user_id = _user_id
  INTO rec_profile;

  IF rec_user.id IS NOT NULL AND rec_profile.user_id IS NOT NULL THEN
    RETURN QUERY SELECT
      rec_user.id as user_id,
      rec_user.nickname,
      rec_user.email,
      rec_user."role",
      rec_profile.avatar,
      rec_profile.descript,
      rec_profile.theme,
      rec_profile.locale,
      rec_user.created_at,
      CASE WHEN rec_user.updated_at > rec_profile.updated_at
        THEN rec_user.updated_at
        ELSE rec_profile.updated_at
      END AS updated_at;
  END IF;
END;
$$;

-- **

/* Create a stored function to modify a user and their profile. */
CREATE OR REPLACE FUNCTION modify_user_profile(
  IN _user_id INTEGER,
  IN _nickname VARCHAR,
  IN _email VARCHAR,
  IN _password VARCHAR,
  IN _role user_role,
  IN _avatar VARCHAR,
  IN _descript TEXT,
  IN _theme VARCHAR,
  IN _locale VARCHAR,
  OUT user_id INTEGER,
  OUT nickname VARCHAR,
  OUT email VARCHAR,
  OUT "role" user_role,
  OUT avatar VARCHAR,
  OUT descript TEXT,
  OUT theme VARCHAR,
  OUT locale VARCHAR,
  OUT created_at TIMESTAMPTZ,
  OUT updated_at TIMESTAMPTZ
) RETURNS SETOF record LANGUAGE plpgsql
AS $$
DECLARE
  is_modify_user BOOLEAN;
  is_modify_profile BOOLEAN;
  rec_user RECORD;
  rec_profile RECORD;
BEGIN
  is_modify_user := _nickname IS NOT NULL OR _email IS NOT NULL OR _password IS NOT NULL _role;
  is_modify_profile := _avatar IS NOT NULL OR _descript IS NOT NULL OR _theme IS NOT NULL OR _locale IS NOT NULL;
  IF _user_id IS NULL OR (NOT is_modify_user AND  NOT is_modify_profile) THEN
    RETURN;
  END IF;

  IF is_modify_user THEN
    UPDATE users SET
      nickname = COALESCE(_nickname, users.nickname),
      email = COALESCE(_email, users.email),
      "role" = COALESCE(_role, users."role")
    WHERE users.id = _user_id
    RETURNING
      users.id, users.nickname, users.email, users."role", users.created_at, users.updated_at
    INTO rec_user;
  ELSE
    SELECT
      u.id, u.nickname, u.email, u."role", u.created_at, u.updated_at
    FROM users u
    WHERE u.id = _user_id
    INTO rec_user;
  END IF;

  IF is_modify_profile THEN
    UPDATE profiles SET
      avatar = COALESCE(_avatar, profiles.avatar),
      descript = COALESCE(_descript, profiles.descript),
      theme = COALESCE(_theme, profiles.theme),
      locale = COALESCE(_locale, profiles.locale)
    WHERE profiles.user_id = _user_id
    RETURNING
      profiles.user_id, profiles.avatar, profiles.descript, profiles.theme, profiles.locale, profiles.updated_at
    INTO rec_profile;
  ELSE
    SELECT
      p.user_id, p.avatar, p.descript, p.theme, p.locale, p.updated_at
    FROM profiles p
    WHERE p.user_id = _user_id
    INTO rec_profile;
  END IF;

  IF rec_user.id IS NOT NULL AND rec_profile.user_id IS NOT NULL THEN
    RETURN QUERY SELECT
      rec_user.id AS user_id, rec_user.nickname, rec_user.email, rec_user."role", 
      rec_profile.avatar, rec_profile.descript, rec_profile.theme, rec_profile.locale, 
      rec_user.created_at,
      CASE WHEN rec_user.updated_at > rec_profile.updated_at
        THEN rec_user.updated_at
        ELSE rec_profile.updated_at
      END AS updated_at;
  END IF;
END;
$$;

-- **
