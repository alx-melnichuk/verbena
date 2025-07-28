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
    date_created TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    /* Date and time the message was changed. */
    date_changed TIMESTAMP WITH TIME ZONE NULL,
    /* Date and time the message was removed. */
    date_removed TIMESTAMP WITH TIME ZONE NULL
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
    date_update TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE INDEX idx_chat_message_logs_chat_message_id ON chat_message_logs(chat_message_id);

-- **

/* Create a stored function that will filter "chat_message" entities by the specified parameters. */
CREATE OR REPLACE FUNCTION filter_chat_messages(
  IN _stream_id INTEGER,
  IN _sort_des BOOLEAN,
  IN _min_date_created TIMESTAMP WITH TIME ZONE,
  IN _max_date_created TIMESTAMP WITH TIME ZONE,
  IN _rec_limit INTEGER,
  OUT id INTEGER,
  OUT stream_id INTEGER,
  OUT user_id INTEGER,
  OUT user_name VARCHAR,
  OUT msg VARCHAR,
  OUT date_created TIMESTAMP WITH TIME ZONE,
  OUT date_changed TIMESTAMP WITH TIME ZONE,
  OUT date_removed TIMESTAMP WITH TIME ZONE
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
  OUT date_created TIMESTAMP WITH TIME ZONE,
  OUT date_changed TIMESTAMP WITH TIME ZONE,
  OUT date_removed TIMESTAMP WITH TIME ZONE
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
  OUT date_created TIMESTAMP WITH TIME ZONE,
  OUT date_changed TIMESTAMP WITH TIME ZONE,
  OUT date_removed TIMESTAMP WITH TIME ZONE
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
  OUT date_created TIMESTAMP WITH TIME ZONE,
  OUT date_changed TIMESTAMP WITH TIME ZONE,
  OUT date_removed TIMESTAMP WITH TIME ZONE
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
  OUT date_update TIMESTAMP WITH TIME ZONE
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
    user_id INT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    /* The user who was blocked. */
    blocked_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    /* Date and time the blocking started. */
    block_date TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE INDEX idx_blocked_users_user_id ON blocked_users(user_id);
CREATE UNIQUE INDEX uq_idx_blocked_users_blocked_id_user_id ON blocked_users(blocked_id, user_id);

-- **

/* Create a stored function to add a new entry to "blocked_users". */
CREATE OR REPLACE FUNCTION create_blocked_user(
  IN _user_id INTEGER,
  IN _blocked_id INTEGER,
  IN _blocked_nickname VARCHAR,
  OUT id INTEGER,
  OUT user_id INTEGER,
  OUT blocked_id INTEGER,
  OUT blocked_nickname VARCHAR,
  OUT block_date TIMESTAMP WITH TIME ZONE
) RETURNS SETOF record LANGUAGE plpgsql
AS $$
DECLARE 
  rec1 RECORD;
  bl_id INTEGER;
  bl_nickname VARCHAR;
BEGIN
  IF (_user_id IS NULL) THEN
    RETURN;
  END IF;

  IF _blocked_id IS NOT NULL THEN 
    SELECT u.id, u.nickname
    FROM users u 
    WHERE u.id = _blocked_id
    INTO bl_id, bl_nickname;
  ELSE
    IF _blocked_nickname IS NOT NULL THEN 
      SELECT u.id, u.nickname 
      FROM users u 
      WHERE u.nickname = _blocked_nickname
      INTO bl_id, bl_nickname;
    END IF;
  END IF;

  IF (bl_id IS NULL OR bl_nickname IS NULL) THEN
    RETURN;
  END IF;

  -- Check for the presence of such a record.
  SELECT
    blocked_users.id,
    blocked_users.user_id,
    blocked_users.blocked_id,
    blocked_users.block_date
  FROM blocked_users
  WHERE blocked_users.user_id = _user_id AND blocked_users.blocked_id = bl_id
  INTO rec1;

  -- If there is no such entry, add it.
  IF rec1.id IS NULL THEN
    -- Add a new entry to the "blocked_user" table.
    INSERT INTO blocked_users(user_id, blocked_id)
    VALUES (_user_id, bl_id)
    RETURNING
      blocked_users.id,
      blocked_users.user_id,
      blocked_users.blocked_id,
      blocked_users.block_date
    INTO rec1;
  END IF;

  IF rec1.id IS NULL THEN
    RETURN;
  END IF;

  RETURN QUERY SELECT
    rec1.id,
    rec1.user_id,
    rec1.blocked_id,
    bl_nickname as blocked_nickname,
    rec1.block_date;
END;
$$;

/* Create a stored function to delete the entity in "blocked_users". */
CREATE OR REPLACE FUNCTION delete_blocked_user(
  IN _user_id INTEGER,
  IN _blocked_id INTEGER,
  IN _blocked_nickname VARCHAR,
  OUT id INTEGER,
  OUT user_id INTEGER,
  OUT blocked_id INTEGER,
  OUT blocked_nickname VARCHAR,
  OUT block_date TIMESTAMP WITH TIME ZONE
) RETURNS SETOF record LANGUAGE plpgsql
AS $$
DECLARE
  rec1 RECORD;
  bl_id INTEGER;
  bl_nickname VARCHAR;
BEGIN
  IF (_user_id IS NULL) THEN
    RETURN;
  END IF;

  IF _blocked_id IS NOT NULL THEN 
    SELECT u.id, u.nickname
    FROM users u 
    WHERE u.id = _blocked_id
    INTO bl_id, bl_nickname;
  ELSE
    IF _blocked_nickname IS NOT NULL THEN 
      SELECT u.id, u.nickname 
      FROM users u 
      WHERE u.nickname = _blocked_nickname
      INTO bl_id, bl_nickname;
    END IF;
  END IF;

  IF (bl_id IS NULL OR bl_nickname IS NULL)  THEN
    RETURN;
  END IF;

  DELETE FROM blocked_users
  WHERE blocked_users.user_id = _user_id
    AND blocked_users.blocked_id = bl_id
  RETURNING 
    blocked_users.id,
    blocked_users.user_id,
    blocked_users.blocked_id,
    blocked_users.block_date
  INTO rec1;

  IF rec1.id IS NULL THEN
    RETURN;
  END IF;

  RETURN QUERY SELECT
    rec1.id,
    rec1.user_id,
    rec1.blocked_id,
    bl_nickname as blocked_nickname,
    rec1.block_date;
END;
$$;

/* Create a stored function that will get the list of "blocked_user" by the specified parameter. */
CREATE OR REPLACE FUNCTION get_blocked_users(
  IN _user_id INTEGER,
  IN _stream_id INTEGER,
  OUT id INTEGER,
  OUT user_id INTEGER,
  OUT blocked_id INTEGER,
  OUT blocked_nickname VARCHAR,
  OUT block_date TIMESTAMP WITH TIME ZONE
) RETURNS SETOF record LANGUAGE plpgsql
AS $$
DECLARE
  stream_id2 INTEGER;
  rec1 RECORD;
  bl_id INTEGER;
  bl_nickname VARCHAR;
BEGIN
  IF (_user_id IS NULL OR _stream_id IS NULL) THEN
    RETURN;
  END IF;

  SELECT s.id
  FROM streams s
  WHERE s.id = _stream_id AND s.user_id = _user_id
  INTO stream_id2;

  IF (stream_id2 IS NULL) THEN
    -- If the user is not the owner of the stream, the result is an empty array.
    RETURN;
  END IF;

  RETURN QUERY
    SELECT
      bu.id,
      bu.user_id,
      bu.blocked_id,
      u.nickname AS blocked_nickname,
      bu.block_date
    FROM
      blocked_users bu, users u
    WHERE
      bu.user_id = _user_id
      AND bu.blocked_id = u.id
    ORDER BY
      u.nickname ASC;
END;
$$;


/* Create a stored function to get information about the live of the stream. */
CREATE OR REPLACE FUNCTION get_stream_live(
  IN _stream_id INTEGER,
  OUT stream_id INTEGER,
  OUT stream_live BOOLEAN
) RETURNS SETOF record LANGUAGE plpgsql
AS $$
DECLARE 
  rec1 RECORD;
BEGIN
  IF (_stream_id IS NULL) THEN
    RETURN;
  END IF;

  SELECT s.id AS stream_id, s.live AS stream_live
  FROM streams s 
  WHERE s.id = _stream_id
  INTO rec1;
 
  IF rec1.stream_live IS NULL THEN 
    RETURN;
  END IF;

  RETURN QUERY SELECT
    rec1.stream_id,
    rec1.stream_live;
END;
$$;

/* Create a stored function to get chat access information. (ChatAccess) */
CREATE OR REPLACE FUNCTION get_chat_access(
  IN _stream_id INTEGER,
  IN _user_id INTEGER,
  OUT stream_id INTEGER,
  OUT stream_owner INTEGER,
  OUT stream_live BOOLEAN,
  OUT is_blocked BOOLEAN
) RETURNS SETOF record LANGUAGE plpgsql
AS $$
DECLARE 
  rec1 RECORD;
  blocked_id INTEGER;
BEGIN
  IF (_stream_id IS NULL OR _user_id IS NULL) THEN
    RETURN;
  END IF;

  SELECT s.id AS stream_id, s.user_id AS stream_owner, s.live AS stream_live
  FROM streams s 
  WHERE s.id = _stream_id
  INTO rec1;
 
  IF rec1.stream_id IS NULL THEN 
    RETURN;
  END IF;

  SELECT bu.id
  FROM blocked_users bu 
  WHERE bu.user_id = rec1.stream_owner AND bu.blocked_id = _user_id
  INTO blocked_id;

  RETURN QUERY SELECT
    rec1.stream_id,
    rec1.stream_owner,
    rec1.stream_live,
    CASE WHEN rec1.stream_owner = _user_id THEN FALSE ELSE (blocked_id IS NOT NULL) END AS is_blocked;
END;
$$;

-- **

/* Create a procedure that adds test data to the table: chat messages, chat_message logs. */
CREATE OR REPLACE PROCEDURE add_chat_messages_test_data()
LANGUAGE plpgsql 
AS $$
DECLARE
  names VARCHAR[];
  nickname1 VARCHAR;
  len1 INTEGER;
  idx1 INTEGER;
  rec1 record;
  mark_ids INTEGER[] := ARRAY[]::INTEGER[];
  stream_ids INTEGER[] := ARRAY[]::INTEGER[];
  user_ids INTEGER[] := ARRAY[]::INTEGER[];
  starttimes TIMESTAMP WITH TIME ZONE[] := ARRAY[]::TIMESTAMP WITH TIME ZONE[];
  len2 INTEGER;
  idx2 INTEGER;
  usr_len INTEGER;
  usr_idx INTEGER;
  mark_id INTEGER;
  stream_id INTEGER;
  user_id INTEGER;
  starttime TIMESTAMP WITH TIME ZONE;
  msg1 VARCHAR;
  ch_msg_id INTEGER;
  ch_msg_logs_ids INTEGER[];
BEGIN
  -- raise notice 'Start';
  names := ARRAY['Ethan_Brown' , 'Ava_Wilson'   , 'James_Miller'   , 'Mila_Davis'  , 'evelyn_allen'];

  len1 := ARRAY_LENGTH(names, 1);
  idx1 := 1;
    WHILE idx1 <= len1 LOOP
      nickname1 = LOWER(names[idx1]);
      -- raise notice '_';
      -- raise notice 'idx1: %, nickname1: %', idx1, nickname1;

      FOR rec1 IN
        SELECT s.id AS stream_id, s.user_id AS user_id, s.starttime AS starttime
        FROM streams s, users u
        WHERE s.user_id = u.id AND s.starttime < now() AND u.nickname = nickname1
        ORDER BY s.starttime ASC
        LIMIT 6 -- Get 6 streams for each user.
      LOOP
        mark_id := rec1.stream_id;
        stream_ids := stream_ids || rec1.stream_id;
        IF rec1.user_id <> ALL(user_ids) THEN
          user_ids := user_ids || rec1.user_id;
        END IF;
        starttimes := starttimes || rec1.starttime;
      END LOOP;
      mark_ids := mark_ids || mark_id;
      idx1 := idx1 + 1;
    END LOOP;

    -- raise notice '_';
    -- raise notice 'stream_ids: %, LEN(stream_ids): %', stream_ids, ARRAY_LENGTH(stream_ids, 1);
    -- raise notice 'user_ids: %, LEN(user_ids): %', user_ids, ARRAY_LENGTH(user_ids, 1);
    -- raise notice 'mark_ids: %, LEN(mark_ids): %', mark_ids, ARRAY_LENGTH(mark_ids, 1);
    len1 := ARRAY_LENGTH(mark_ids, 1);
    IF len1 >= 2 THEN
      mark_ids := ARRAY[]::INTEGER[] || mark_ids[len1 - 1] || mark_ids[len1];
    END IF;
    -- raise notice '_';
    usr_len := ARRAY_LENGTH(user_ids, 1);
    len1 := ARRAY_LENGTH(stream_ids, 1);
    idx1 := 1;
    WHILE idx1 <= len1 LOOP
      stream_id := stream_ids[idx1];
      usr_idx := 1;
      len2 := CASE WHEN stream_id = mark_id THEN 140 ELSE 15 END;
      idx2 := 1;
      WHILE idx2 <= len2 LOOP
        starttime := (starttimes[idx1] + (idx2 * INTERVAL '1 hours'))::timestamp;
        msg1 := 'Demo message ' || idx2;
        user_id := user_ids[usr_idx];

        -- Add a new message for the specified user and their stream.
        INSERT INTO chat_messages(stream_id, user_id, msg, date_created)
        SELECT stream_id, user_id, msg1, starttime
        RETURNING chat_messages.id
        INTO ch_msg_id;
        -- raise notice 'ch_msg_id: %, stream_id: %, user_id: %, msg1: %, starttime: %', ch_msg_id, stream_id, user_id, msg1, starttime;

        IF MOD(ch_msg_id, 2) = 0  THEN
          -- Add message change.
          ch_msg_logs_ids := ARRAY(SELECT id FROM modify_chat_message(ch_msg_id, user_id, msg1 || ' ver.2'));
        ELSE 
          IF MOD(ch_msg_id, 9) = 0  THEN
            -- Delete message contents.
            ch_msg_logs_ids := ARRAY(SELECT id FROM modify_chat_message(ch_msg_id, user_id, ''));
          END IF;
        END IF;

        usr_idx := CASE WHEN usr_idx = usr_len THEN 1 ELSE usr_idx + 1 END;
        idx2 := idx2 + 1;
      END LOOP;
      idx1 := idx1 + 1;
    END LOOP;

  -- raise notice 'Finish';
END;
$$;

/*
 * Add test data to the tables: chat_messages, chat_message_logs.
 */
CALL add_chat_messages_test_data();

/* Removing the procedure that adds test data to the table: chat messages, chat_message logs. */
DROP PROCEDURE IF EXISTS add_chat_messages_test_data;

-- **

/* Create a procedure that adds test data to the table: blocked_users. */
CREATE OR REPLACE PROCEDURE add_blocked_users_test_data()
LANGUAGE plpgsql 
AS $$
DECLARE
  names VARCHAR[];
  nameIds INTEGER[];
  nickname1 VARCHAR;
  len1 INTEGER;
  idx1 INTEGER;
  user_id1 INTEGER;
  user_id2 INTEGER;
BEGIN
  -- raise notice 'Start';
  names := ARRAY['ethan_brown', 'ava_wilson', 'james_miller', 'mila_davis', 'evelyn_allen'];

  SELECT array_agg(u.id)
  FROM users u
  WHERE u.nickname IN (SELECT unnest(names))
  INTO nameIds;
  -- raise notice 'LEN(nameIds): %, nameIds: %', ARRAY_LENGTH(nameIds, 1), nameIds;

  len1 := ARRAY_LENGTH(nameIds, 1);
  user_id1 = nameIds[1];
  idx1 := 2;
  WHILE idx1 <= len1 LOOP
    user_id2 = nameIds[idx1];
    PERFORM create_blocked_user(user_id1, user_id2, NULL);
    user_id1 = user_id2;
    idx1 := idx1 + 1;
  END LOOP;

  IF (len1 > 1) THEN
    PERFORM create_blocked_user(nameIds[len1], nameIds[1], NULL);
  END IF;
  -- raise notice 'Finish';
END;
$$;

/*
 * Add test data to the tables: blocked_users.
 */
CALL add_blocked_users_test_data();

/* Removing the procedure that adds test data to the table: blocked_users. */
DROP PROCEDURE IF EXISTS add_blocked_users_test_data;

-- **

/* Create a stored function that will filter "stream" entities by the specified parameters. */
CREATE OR REPLACE FUNCTION filter_streams(
  IN _id INTEGER,
  IN _user_id INTEGER,
  IN _is_logo BOOLEAN,
  IN _is_live BOOLEAN,
  OUT id INTEGER,
  OUT user_id INTEGER,
  OUT title VARCHAR,
  OUT descript TEXT,
  OUT logo VARCHAR,
  OUT starttime TIMESTAMP WITH TIME ZONE,
  OUT live BOOLEAN,
  OUT state stream_state,
  OUT started TIMESTAMP WITH TIME ZONE,
  OUT stopped TIMESTAMP WITH TIME ZONE,
  OUT source VARCHAR,
  OUT created_at TIMESTAMP WITH TIME ZONE,
  OUT updated_at TIMESTAMP WITH TIME ZONE
) RETURNS SETOF record LANGUAGE plpgsql
AS $$
BEGIN
  IF _id IS NULL AND _user_id IS NULL THEN
    RETURN;
  END IF;

  RETURN QUERY 
    SELECT s.id, s.user_id, s.title, s.descript, s.logo, s.starttime, s.live, s.state,
      s.started, s.stopped, s.source, s.created_at, s.updated_at
    FROM streams s 
    WHERE s.id = COALESCE(_id, s.id)
      AND s.user_id = COALESCE(_user_id, s.user_id)
      AND CASE WHEN _is_logo = true THEN LENGTH(COALESCE(s.logo, '')) > 0
          ELSE CASE WHEN _is_logo = false THEN LENGTH(COALESCE(s.logo, '')) = 0 ELSE true END
          END
      AND s.live = COALESCE(_is_live, s.live)
    ORDER BY s.id ASC;

END;
$$;
