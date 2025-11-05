-- Adding entities: "chat_messages", chat_message_logs.

-- **

/* Create "chat_messages" table. */
CREATE TABLE chat_messages (
    id SERIAL PRIMARY KEY NOT NULL,
    /* Attached to the entity. */
    stream_id INTEGER NOT NULL REFERENCES streams(id) ON DELETE CASCADE,
    /* Owner id */
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    /* Message text. */
    msg VARCHAR(255) NULL,
    /* Date and time of message creation. */
    date_created TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    /* Date and time the message was changed. */
    date_changed TIMESTAMPTZ NULL,
    /* Date and time the message was removed. */
    date_removed TIMESTAMPTZ NULL
);

CREATE INDEX idx_chat_messages_stream_id ON chat_messages(stream_id);
CREATE INDEX idx_chat_messages_user_id ON chat_messages(user_id);
CREATE INDEX idx_chat_messages_date_created ON chat_messages(date_created);

-- **

/* Create "chat_message_logs" table. */
CREATE TABLE chat_message_logs (
    id SERIAL PRIMARY KEY NOT NULL,
    /* Owner id */
    chat_message_id INTEGER NOT NULL REFERENCES chat_messages(id) ON DELETE CASCADE,
    /* Old message value. */
    old_msg VARCHAR(255) NOT NULL,
    /* Date and time of message creation/modification. */
    date_update TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE INDEX idx_chat_message_logs_chat_message_id ON chat_message_logs(chat_message_id);

-- **

/* Create a stored function that will filter "chat_message" entities by the specified parameters. */
CREATE OR REPLACE FUNCTION filter_chat_messages(
  IN _stream_id INTEGER,
  IN _sort_des BOOLEAN,
  IN _min_date_created TIMESTAMPTZ,
  IN _max_date_created TIMESTAMPTZ,
  IN _rec_limit INTEGER,
  OUT id INTEGER,
  OUT stream_id INTEGER,
  OUT user_id INTEGER,
  OUT user_name VARCHAR,
  OUT msg VARCHAR,
  OUT date_created TIMESTAMPTZ,
  OUT date_changed TIMESTAMPTZ,
  OUT date_removed TIMESTAMPTZ
) RETURNS SETOF record LANGUAGE plpgsql
AS $$
DECLARE
BEGIN
    IF _min_date_created IS NULL THEN
      _min_date_created := TO_TIMESTAMP(0);
    END IF;  
  IF _max_date_created IS NULL THEN
    _max_date_created := CURRENT_TIMESTAMP;
  END IF;
  IF _rec_limit IS NULL THEN
    _rec_limit := 20;
  END IF;
  
  IF _sort_des THEN
    RETURN QUERY
      SELECT cm.id, cm.stream_id, cm.user_id, u.nickname as user_name, cm.msg,
        cm.date_created, cm.date_changed, cm.date_removed
      FROM chat_messages cm, users u
      WHERE cm.stream_id = _stream_id
        AND u.id = cm.user_id
        AND _min_date_created < cm.date_created
        AND cm.date_created < _max_date_created
      ORDER BY cm.date_created DESC
      LIMIT _rec_limit;
  ELSE
    RETURN QUERY
      SELECT cm.id, cm.stream_id, cm.user_id, u.nickname as user_name, cm.msg,
        cm.date_created, cm.date_changed, cm.date_removed
      FROM chat_messages cm, users u
      WHERE cm.stream_id = _stream_id
        AND u.id = cm.user_id
        AND _min_date_created < cm.date_created
        AND cm.date_created < _max_date_created
      ORDER BY cm.date_created ASC
      LIMIT _rec_limit;
  END IF;
END;
$$;

/* Create a stored function to add a new entry to "chat_messages". */
CREATE OR REPLACE FUNCTION create_chat_message(
  IN _stream_id INTEGER,
  IN _user_id INTEGER,
  IN _msg VARCHAR,
  OUT id INTEGER,
  OUT stream_id INTEGER,
  OUT user_id INTEGER,
  OUT user_name VARCHAR,
  OUT msg VARCHAR,
  OUT date_created TIMESTAMPTZ,
  OUT date_changed TIMESTAMPTZ,
  OUT date_removed TIMESTAMPTZ
) RETURNS SETOF record LANGUAGE plpgsql
AS $$
DECLARE 
  rec1 RECORD;
BEGIN
  -- Add a new entry to the "chat_messages" table.
  INSERT INTO chat_messages(stream_id, user_id, msg)
  VALUES (_stream_id, _user_id, _msg)
  RETURNING
    chat_messages.id, chat_messages.stream_id, chat_messages.user_id, chat_messages.msg,
    chat_messages.date_created, chat_messages.date_changed, chat_messages.date_removed
  INTO rec1;

  IF rec1.id IS NULL THEN
    RETURN;
  END IF;

  SELECT u.nickname FROM users u WHERE u.id = _user_id INTO user_name;

  RETURN QUERY SELECT
    rec1.id, rec1.stream_id, rec1.user_id, user_name, rec1.msg,
    rec1.date_created, rec1.date_changed, rec1.date_removed;
END;
$$;


/* Create a stored function to modify the entry in "chat_messages". */
CREATE OR REPLACE FUNCTION modify_chat_message(
  IN _id INTEGER,
  IN _user_id INTEGER,
  IN _msg VARCHAR,
  OUT id INTEGER,
  OUT stream_id INTEGER,
  OUT user_id INTEGER,
  OUT user_name VARCHAR,
  OUT msg VARCHAR,
  OUT date_created TIMESTAMPTZ,
  OUT date_changed TIMESTAMPTZ,
  OUT date_removed TIMESTAMPTZ
) RETURNS SETOF record LANGUAGE plpgsql
AS $$
DECLARE
  rec1 RECORD;
  is_changed BOOLEAN;
BEGIN
  IF (_id IS NULL OR _user_id IS NULL) THEN
    RETURN;
  END IF;

  IF _msg IS NOT NULL THEN
    INSERT INTO chat_message_logs (chat_message_id, old_msg, date_update)
    SELECT chat_messages.id, chat_messages.msg, CURRENT_TIMESTAMP
    FROM chat_messages
    WHERE chat_messages.date_removed IS NULL
      AND chat_messages.id = _id
      AND chat_messages.user_id = _user_id;
  END IF;

  is_changed := _msg IS NOT NULL AND LENGTH(_msg) > 0;
  
  UPDATE chat_messages SET
    msg = _msg,
    date_changed = CASE WHEN is_changed THEN CURRENT_TIMESTAMP ELSE chat_messages.date_changed END,
    date_removed = CASE WHEN (NOT is_changed) THEN CURRENT_TIMESTAMP ELSE chat_messages.date_removed END
  WHERE chat_messages.date_removed IS NULL
    AND chat_messages.id = _id
    AND chat_messages.user_id = _user_id
  RETURNING
    chat_messages.id, chat_messages.stream_id, chat_messages.user_id, chat_messages.msg,
    chat_messages.date_created, chat_messages.date_changed, chat_messages.date_removed
  INTO rec1;

  IF rec1.id IS NULL THEN
    RETURN;
  END IF;

  SELECT u.nickname FROM users u WHERE u.id = _user_id INTO user_name;

  RETURN QUERY SELECT
    rec1.id, rec1.stream_id, rec1.user_id, user_name, rec1.msg,
    rec1.date_created, rec1.date_changed, rec1.date_removed;
END;
$$;


/* Create a stored function to delete the entity in "chat_messages". */
CREATE OR REPLACE FUNCTION delete_chat_message(
  IN _id INTEGER,
  IN _user_id INTEGER,
  OUT id INTEGER,
  OUT stream_id INTEGER,
  OUT user_id INTEGER,
  OUT user_name VARCHAR,
  OUT msg VARCHAR,
  OUT date_created TIMESTAMPTZ,
  OUT date_changed TIMESTAMPTZ,
  OUT date_removed TIMESTAMPTZ
) RETURNS SETOF record LANGUAGE plpgsql
AS $$
DECLARE
  rec1 RECORD;
BEGIN
  IF _id IS NULL OR _user_id IS NULL THEN
    RETURN;
  END IF;

  DELETE FROM chat_messages
  WHERE chat_messages.id = _id
    AND chat_messages.user_id = _user_id
  RETURNING 
    chat_messages.id, chat_messages.stream_id, chat_messages.user_id, chat_messages.msg,
    chat_messages.date_created, chat_messages.date_changed, chat_messages.date_removed
  INTO rec1;

  IF rec1.id IS NULL THEN
    RETURN;
  END IF;

  SELECT u.nickname FROM users u WHERE u.id = _user_id INTO user_name;

  RETURN QUERY SELECT
    rec1.id, rec1.stream_id, rec1.user_id, user_name, rec1.msg,
    rec1.date_created, rec1.date_changed, rec1.date_removed;
END;
$$;

-- **

/* Create a stored function to get an array of entities in "chat_message_logs". */
CREATE OR REPLACE FUNCTION get_chat_message_log(
  IN _chat_message_id INTEGER,
  OUT id INTEGER,
  OUT chat_message_id INTEGER,
  OUT old_msg VARCHAR,
  OUT date_update TIMESTAMPTZ
) RETURNS SETOF record LANGUAGE sql
AS $$
  SELECT
    chat_message_logs.id, chat_message_logs.chat_message_id,
    chat_message_logs.old_msg, chat_message_logs.date_update
  FROM
    chat_message_logs
  WHERE
    chat_message_logs.chat_message_id = _chat_message_id
  ORDER BY
    chat_message_logs.id ASC;
$$;

-- **

/* Create "blocked_users" table. */
CREATE TABLE blocked_users (
    id SERIAL PRIMARY KEY NOT NULL,
    /* The user who performed the blocking. */
    owner_id INT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    /* The user who was blocked. */
    blocked_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    /* Date and time the blocking started. */
    block_date TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE INDEX idx_blocked_users_owner_id ON blocked_users(owner_id);
CREATE UNIQUE INDEX uq_idx_blocked_users_blocked_id_owner_id ON blocked_users(blocked_id, owner_id);
CREATE INDEX idx_blocked_users_block_date ON blocked_users(block_date);

-- **

/* Create a stored function to add a new entry to "blocked_users". */
CREATE OR REPLACE FUNCTION create_blocked_user(
  IN _owner_id INTEGER,
  IN _blocked_id INTEGER,
  IN _blocked_nickname VARCHAR,
  OUT id INTEGER,
  OUT user_id INTEGER,
  OUT nickname VARCHAR,
  OUT block_date TIMESTAMPTZ
) RETURNS SETOF record LANGUAGE plpgsql
AS $$
DECLARE 
  rec1 RECORD;
  bl_user_id INTEGER;
  bl_nickname VARCHAR;
BEGIN
  IF (_owner_id IS NULL) THEN
    RETURN;
  END IF;

  IF _blocked_id IS NOT NULL THEN 
    SELECT u.id, u.nickname
    FROM users u 
    WHERE u.id = _blocked_id
    INTO bl_user_id, bl_nickname;
  ELSIF _blocked_nickname IS NOT NULL THEN 
    SELECT u.id, u.nickname 
    FROM users u 
    WHERE u.nickname = _blocked_nickname
    INTO bl_user_id, bl_nickname;
  END IF;

  IF (bl_user_id IS NULL OR bl_nickname IS NULL) THEN
    RETURN;
  END IF;

  -- Check for the presence of such a record.
  SELECT
    blocked_users.id,
    blocked_users.owner_id,
    blocked_users.blocked_id,
    blocked_users.block_date
  FROM blocked_users
  WHERE blocked_users.owner_id = _owner_id AND blocked_users.blocked_id = bl_user_id
  INTO rec1;

  -- If there is no such entry, add it.
  IF rec1.id IS NULL THEN
    -- Add a new entry to the "blocked_user" table.
    INSERT INTO blocked_users(owner_id, blocked_id)
    VALUES (_owner_id, bl_user_id)
    RETURNING
      blocked_users.id,
      blocked_users.blocked_id,
      blocked_users.block_date
    INTO rec1;
  END IF;

  IF rec1.id IS NULL THEN
    RETURN;
  END IF;

  RETURN QUERY SELECT
    rec1.id,
    rec1.blocked_id AS user_id,
    bl_nickname as nickname,
    rec1.block_date;
END;
$$;

/* Create a stored function to delete the entity in "blocked_users". */
CREATE OR REPLACE FUNCTION delete_blocked_user(
  IN _owner_id INTEGER,
  IN _blocked_id INTEGER,
  IN _blocked_nickname VARCHAR,
  OUT id INTEGER,
  OUT user_id INTEGER,
  OUT nickname VARCHAR,
  OUT block_date TIMESTAMPTZ
) RETURNS SETOF record LANGUAGE plpgsql
AS $$
DECLARE
  rec1 RECORD;
  bl_user_id INTEGER;
  bl_nickname VARCHAR;
BEGIN
  IF (_owner_id IS NULL) THEN
    RETURN;
  END IF;

  IF _blocked_id IS NOT NULL THEN 
    SELECT u.id, u.nickname
    FROM users u 
    WHERE u.id = _blocked_id
    INTO bl_user_id, bl_nickname;
  ELSIF _blocked_nickname IS NOT NULL THEN 
    SELECT u.id, u.nickname 
    FROM users u 
    WHERE u.nickname = _blocked_nickname
    INTO bl_user_id, bl_nickname;
  END IF;

  IF (bl_user_id IS NULL OR bl_nickname IS NULL)  THEN
    RETURN;
  END IF;

  DELETE FROM blocked_users
  WHERE blocked_users.owner_id = _owner_id
    AND blocked_users.blocked_id = bl_user_id
  RETURNING 
    blocked_users.id,
    blocked_users.blocked_id,
    blocked_users.block_date
  INTO rec1;

  IF rec1.id IS NULL THEN
    RETURN;
  END IF;

  RETURN QUERY SELECT
    rec1.id,
    rec1.blocked_id AS user_id,
    bl_nickname as nickname,
    rec1.block_date;
END;
$$;

/* Create a stored function that will get the list of "blocked_user" by the specified parameter. */
CREATE OR REPLACE FUNCTION get_blocked_nicknames(
  IN _owner_id INTEGER,
  OUT id INTEGER,
  OUT blocked_id INTEGER,
  OUT nickname VARCHAR
) RETURNS SETOF record LANGUAGE plpgsql
AS $$
BEGIN
  IF (_owner_id IS NULL) THEN
    RETURN;
  END IF;

  RETURN QUERY
    SELECT
      bu.id,
      bu.blocked_id,
      u.nickname
    FROM
      blocked_users bu, users u
    WHERE
      bu.owner_id = _owner_id
      AND bu.blocked_id = u.id;
END;
$$;

/* Create a stored function that will get a sorted list of "blocked_user" by the specified parameter. */
CREATE OR REPLACE FUNCTION get_blocked_users_sort(
  IN _owner_id INTEGER,
  IN _sort_column VARCHAR, -- 'nickname','email','block_date'
  IN _sort_desc BOOLEAN,
  OUT id INTEGER,
  OUT user_id INTEGER,
  OUT nickname VARCHAR,
  OUT email VARCHAR,
  OUT block_date TIMESTAMPTZ
) RETURNS SETOF record LANGUAGE plpgsql
AS $$
BEGIN
  IF (_owner_id IS NULL) THEN
    RETURN;
  END IF;
  IF (_sort_desc IS NULL) THEN
    _sort_desc := FALSE;
  END IF;

  IF (_sort_column = 'email') THEN
    RETURN QUERY
      SELECT b.id, b.blocked_id AS user_id, u.nickname, u.email, b.block_date
      FROM blocked_users b, users u
      WHERE b.owner_id = _owner_id AND b.blocked_id = u.id
      ORDER BY 
        CASE WHEN NOT _sort_desc THEN u.email ELSE NULL END ASC,
        CASE WHEN _sort_desc     THEN u.email ELSE NULL END DESC;
  ELSIF (_sort_column = 'block_date') THEN
    RETURN QUERY
      SELECT b.id, b.blocked_id AS user_id, u.nickname, u.email, b.block_date
      FROM blocked_users b, users u
      WHERE b.owner_id = _owner_id AND b.blocked_id = u.id
      ORDER BY 
        CASE WHEN NOT _sort_desc THEN b.block_date ELSE NULL END ASC,
        CASE WHEN _sort_desc     THEN b.block_date ELSE NULL END DESC;
  ELSE
    RETURN QUERY
      SELECT b.id, b.blocked_id AS user_id, u.nickname, u.email, b.block_date
      FROM blocked_users b, users u
      WHERE b.owner_id = _owner_id AND b.blocked_id = u.id
      ORDER BY 
        CASE WHEN NOT _sort_desc THEN u.nickname ELSE NULL END ASC,
        CASE WHEN _sort_desc     THEN u.nickname ELSE NULL END DESC;
  END IF;
END;
$$;

/* Create a stored function that will get the list of "blocked_user" by the specified parameter. */
CREATE OR REPLACE FUNCTION get_blocked_users(
  IN _owner_id INTEGER,
  IN _sort_column VARCHAR, -- 'nickname','email','block_date'
  IN _sort_desc BOOLEAN,
  OUT id INTEGER,
  OUT user_id INTEGER,
  OUT nickname VARCHAR,
  OUT email VARCHAR,
  OUT block_date TIMESTAMPTZ,
  OUT avatar VARCHAR
) RETURNS SETOF record LANGUAGE plpgsql
AS $$
DECLARE
BEGIN
  IF (_owner_id IS NULL) THEN
    RETURN;
  END IF;

  RETURN QUERY
    SELECT
      b.id, b.user_id, b.nickname, b.email, b.block_date, p.avatar
    FROM
      get_blocked_users_sort(_owner_id, _sort_column, _sort_desc) b, profiles p
    WHERE
      b.user_id = p.user_id;
END;
$$;

-- **

/* Create a stored function to get chat access information. (ChatAccess) */
CREATE OR REPLACE FUNCTION get_chat_access(
  IN _stream_id INTEGER,
  IN _user_id INTEGER,
  OUT stream_id INTEGER,
  OUT stream_owner INTEGER,
  OUT stream_state VARCHAR,
  OUT is_blocked BOOLEAN
) RETURNS SETOF record LANGUAGE plpgsql
AS $$
DECLARE
  rec1 RECORD;
  blocked_id INTEGER;
BEGIN
  IF _stream_id IS NULL THEN
    RETURN;
  END IF;

  SELECT s.id AS stream_id, s.user_id AS stream_owner, CAST(s.state AS VARCHAR) AS stream_state
  FROM streams s 
  WHERE s.id = _stream_id
  INTO rec1;

  IF rec1.stream_id IS NULL THEN 
    RETURN;
  END IF;

  IF _user_id IS NOT NULL THEN
    SELECT bu.id
    FROM blocked_users bu 
    WHERE bu.user_id = rec1.stream_owner AND bu.blocked_id = _user_id
    INTO blocked_id;
  ELSE
    blocked_id := -1;
  END IF;

  RETURN QUERY SELECT
    rec1.stream_id,
    rec1.stream_owner,
    rec1.stream_state,
    CASE WHEN rec1.stream_owner = _user_id THEN FALSE 
    ELSE blocked_id IS NOT NULL 
    END AS is_blocked;
END;
$$;

-- **
