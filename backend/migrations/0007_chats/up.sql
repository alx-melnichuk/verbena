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
    /* Date and time of message creation/modification/deletion. */
    date_update TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    /* Flag, message change. */
    is_changed BOOLEAN DEFAULT FALSE NOT NULL,
    /* Flag, message deletion. */
    is_removed BOOLEAN DEFAULT FALSE NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL
);

SELECT diesel_manage_updated_at('chat_messages');

CREATE INDEX idx_chat_messages_stream_id ON chat_messages(stream_id);
CREATE INDEX idx_chat_messages_user_id ON chat_messages(user_id);

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

/* Create a stored function to add a new entry to "chat_messages". */
CREATE OR REPLACE FUNCTION create_chat_message(
  OUT id INTEGER,
  INOUT stream_id INTEGER,
  INOUT user_id INTEGER,
  INOUT msg VARCHAR,
  OUT date_update TIMESTAMP WITH TIME ZONE,
  OUT is_changed BOOLEAN,
  OUT is_removed BOOLEAN,
  OUT created_at TIMESTAMP WITH TIME ZONE,
  OUT updated_at TIMESTAMP WITH TIME ZONE
) RETURNS SETOF record LANGUAGE plpgsql
AS $$
DECLARE 
  rec1 RECORD;
BEGIN
  -- Add a new entry to the "chat_messages" table.
  INSERT INTO chat_messages(stream_id, user_id, msg)
  VALUES (stream_id, user_id, msg)
  RETURNING
    chat_messages.id,
    chat_messages.stream_id,
    chat_messages.user_id,
    chat_messages.msg,
    chat_messages.date_update,
    chat_messages.is_changed,
    chat_messages.is_removed,
    chat_messages.created_at,
    chat_messages.updated_at
    INTO rec1;

  RETURN QUERY SELECT
    rec1.id,
    rec1.stream_id,
    rec1.user_id,
    rec1.msg,
    rec1.date_update,
    rec1.is_changed,
    rec1.is_removed,
    rec1.created_at,
    rec1.updated_at;
END;
$$;


/* Create a stored function to modify the entry in "chat_messages". */
CREATE OR REPLACE FUNCTION modify_chat_message(
  INOUT id INTEGER,
  IN by_user_id INTEGER,
  INOUT stream_id INTEGER,
  INOUT user_id INTEGER,
  INOUT msg VARCHAR,
  OUT date_update TIMESTAMP WITH TIME ZONE,
  OUT is_changed BOOLEAN,
  OUT is_removed BOOLEAN,
  OUT created_at TIMESTAMP WITH TIME ZONE,
  OUT updated_at TIMESTAMP WITH TIME ZONE
) RETURNS SETOF record LANGUAGE plpgsql
AS $$
DECLARE 
  _id INTEGER;
  val1 VARCHAR;
  date_update TIMESTAMP WITH TIME ZONE;
  sql1 TEXT;
  update_fields VARCHAR[];
BEGIN
  _id := id;
  id := NULL;
  IF _id IS NULL THEN
    stream_id := NULL;
    user_id := NULL;
    msg := NULL;
    RETURN;
  END IF;

  update_fields := ARRAY[]::VARCHAR[];

  IF stream_id IS NOT NULL THEN
    update_fields := ARRAY_APPEND(update_fields, 'stream_id = ' || stream_id);
  END IF;

  IF user_id IS NOT NULL THEN
    update_fields := ARRAY_APPEND(update_fields, 'user_id = ' || user_id);
  END IF;

  IF msg IS NOT NULL THEN
    update_fields := ARRAY_APPEND(update_fields, 'msg = ' || '''' || msg || '''');

    update_fields := ARRAY_APPEND(update_fields, 'date_update = CURRENT_TIMESTAMP');

    IF LENGTH(msg) > 0 THEN
      sql1 := 'INSERT INTO chat_message_logs (chat_message_id, old_msg, date_update)'
        || ' SELECT chat_messages.id, chat_messages.msg, chat_messages.date_update'
        || ' FROM chat_messages'
        || ' WHERE chat_messages.is_removed=FALSE'
        || CASE WHEN by_user_id IS NOT NULL THEN ' AND chat_messages.user_id=' || by_user_id ELSE '' END
        || ' AND chat_messages.id=' || _id;

      EXECUTE sql1;

      val1 := 'is_changed = TRUE';
    ELSE
      -- LENGTH(msg) == 0
      val1 := 'is_removed = TRUE';
    END IF;
    update_fields := ARRAY_APPEND(update_fields, val1);
  END IF;

  IF ARRAY_LENGTH(update_fields, 1) > 0 THEN
    sql1 := 'UPDATE chat_messages SET '
      || ARRAY_TO_STRING(update_fields, ',')
      || ' WHERE is_removed=FALSE'
      || CASE WHEN by_user_id IS NOT NULL THEN ' AND user_id=' || by_user_id ELSE '' END
      || ' AND id=' || _id
      || ' RETURNING '
      || ' chat_messages.id, chat_messages.stream_id, chat_messages.user_id, chat_messages.msg,'
      || ' chat_messages.date_update, chat_messages.is_changed, chat_messages.is_removed,'
      || ' chat_messages.created_at, chat_messages.updated_at';

    EXECUTE sql1 INTO
      id, stream_id, user_id, msg, date_update, is_changed, is_removed, created_at, updated_at;
  END IF;

  IF id IS NOT NULL THEN
    RETURN QUERY SELECT
      id, stream_id, user_id, msg, date_update, is_changed, is_removed, created_at, updated_at;
  END IF;
END;
$$;


/* Create a stored function to delete the entity in "chat_messages". */
CREATE OR REPLACE FUNCTION delete_chat_message(
  INOUT id INTEGER,
  OUT stream_id INTEGER,
  OUT user_id INTEGER,
  OUT msg VARCHAR,
  OUT date_update TIMESTAMP WITH TIME ZONE,
  OUT is_changed BOOLEAN,
  OUT is_removed BOOLEAN,
  OUT created_at TIMESTAMP WITH TIME ZONE,
  OUT updated_at TIMESTAMP WITH TIME ZONE
) RETURNS SETOF record LANGUAGE plpgsql
AS $$
DECLARE
  _id INTEGER;
  rec1 RECORD;
BEGIN
  _id := id;
  id := NULL;
  IF _id IS NULL THEN
    RETURN;
  END IF;

  DELETE FROM chat_messages
  WHERE chat_messages.id = _id
  RETURNING 
    chat_messages.id, chat_messages.stream_id, chat_messages.user_id, chat_messages.msg,
    chat_messages.date_update, chat_messages.is_changed, chat_messages.is_removed,
    chat_messages.created_at, chat_messages.updated_at
  INTO rec1;

  IF rec1.id IS NULL THEN
    RETURN;
  END IF;

  RETURN QUERY SELECT
    rec1.id, rec1.stream_id, rec1.user_id, rec1.msg,
    rec1.date_update, rec1.is_changed, rec1.is_removed,
    rec1.created_at, rec1.updated_at;
END;
$$;

-- **

/* Create a stored function to get an array of entities in "chat_message_logs". */
CREATE OR REPLACE FUNCTION get_chat_message_log(
  OUT id INTEGER,
  INOUT chat_message_id INTEGER,
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
    chat_message_logs.chat_message_id = chat_message_id
  ORDER BY
    chat_message_logs.id ASC;
$$;

-- **

-- # Test data
INSERT INTO chat_messages (stream_id,user_id,msg) VALUES
(913,18,'Demo message A v1'),
(913,18,'Demo message B v1'),
(913,18,'Demo message C v1'),
(913,18,'Demo message D v1');

-- **

