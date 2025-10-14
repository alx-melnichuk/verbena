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
  starttimes TIMESTAMPTZ[] := ARRAY[]::TIMESTAMPTZ[];
  len2 INTEGER;
  idx2 INTEGER;
  usr_len INTEGER;
  usr_idx INTEGER;
  mark_id INTEGER;
  stream_id INTEGER;
  user_id INTEGER;
  starttime TIMESTAMPTZ;
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
