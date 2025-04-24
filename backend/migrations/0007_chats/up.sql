/* Create "chat_messages" table. */
CREATE TABLE chat_messages (
    id SERIAL PRIMARY KEY NOT NULL,
    /* Attached to the entity. */
    stream_id INTEGER NOT NULL REFERENCES streams(id) ON DELETE CASCADE,
    /* Owner id */
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    /* Message text. */
    msg VARCHAR(255) NOT NULL,
    /* Date and time the message was created. */
    date_created TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    /* Date and time the message was changed. */
    date_changed TIMESTAMP WITH TIME ZONE NULL,
    /* Date and time the message was deleted. */
    date_removed TIMESTAMP WITH TIME ZONE NULL,
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
    /* New message value. */
    new_msg VARCHAR(255) NOT NULL,
    /* Date and time the message was changed. */
    date_changed TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE INDEX idx_chat_message_logs_chat_message_id ON chat_message_logs(chat_message_id);

-- **


/* Create a stored function to add a new entry to "chat_messages". */
/*CREATE OR REPLACE FUNCTION create_chat_message1(
  IN _stream_id INTEGER,
  IN _user_id INTEGER,
  IN _msg VARCHAR,
  OUT id INTEGER,
  OUT stream_id INTEGER,
  OUT user_id INTEGER,
  OUT msg VARCHAR,
  OUT date_msg TIMESTAMP WITH TIME ZONE,
  OUT "version" NUMERIC(4),
  OUT date_removed TIMESTAMP WITH TIME ZONE,
  OUT created_at TIMESTAMP WITH TIME ZONE,
  OUT updated_at TIMESTAMP WITH TIME ZONE
) RETURNS SETOF record LANGUAGE plpgsql
AS $$
DECLARE 
  rec1 RECORD;
BEGIN
  -- Add a new entry to the "chat_messages" table.
  INSERT INTO chat_messages(stream_id, user_id, msg)
  VALUES (_stream_id, _user_id, _msg)
  RETURNING
    chat_messages.id,
    chat_messages.stream_id,
    chat_messages.user_id,
    chat_messages.msg,
    chat_messages.date_msg,
    chat_messages."version",
    chat_messages.date_removed,
    chat_messages.created_at,
    chat_messages.updated_at
    INTO rec1;

  RETURN QUERY SELECT
    rec1.id,
    rec1.stream_id,
    rec1.user_id,
    rec1.msg,
    rec1.date_msg,
    rec1."version",
    rec1.date_removed,
    rec1.created_at,
    rec1.updated_at;
END;
$$;*/

/* Create a stored function to add a new entry to "chat_messages". */
CREATE OR REPLACE FUNCTION create_chat_message(
  OUT id INTEGER,
  INOUT stream_id INTEGER,
  INOUT user_id INTEGER,
  INOUT msg VARCHAR,
  OUT date_created TIMESTAMP WITH TIME ZONE,
  OUT date_changed TIMESTAMP WITH TIME ZONE,
  OUT date_removed TIMESTAMP WITH TIME ZONE,
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
    chat_messages.date_created,
    chat_messages.date_changed,
    chat_messages.date_removed,
    chat_messages.created_at,
    chat_messages.updated_at
    INTO rec1;

  RETURN QUERY SELECT
    rec1.id,
    rec1.stream_id,
    rec1.user_id,
    rec1.msg,
    rec1.date_created,
    rec1.date_changed,
    rec1.date_removed,
    rec1.created_at,
    rec1.updated_at;

END;
$$;

-- **
