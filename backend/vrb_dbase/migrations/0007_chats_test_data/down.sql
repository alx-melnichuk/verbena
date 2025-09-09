-- **

/* Removing the procedure that adds test data to the table: chat messages, chat_message logs. */
DROP PROCEDURE IF EXISTS add_chat_messages_test_data;

-- **

/* Removing the procedure that adds test data to the table: blocked_users. */
DROP PROCEDURE IF EXISTS add_blocked_users_test_data;

-- **

/*
 * Remove test data.
 */
CREATE OR REPLACE PROCEDURE remove_data_test_chat_messages()
LANGUAGE plpgsql 
AS $$
DECLARE
  idx INTEGER := 0;
  name_list VARCHAR[];
  nick VARCHAR := '';
  user_id2 INTEGER;
BEGIN
  RAISE NOTICE 'Start';
  name_list := ARRAY['ethan_brown', 'ava_wilson', 'james_miller', 'mila_davis', 'evelyn_allen'];
   
  idx := ARRAY_LENGTH(name_list, 1);
  WHILE idx > 0 LOOP
    nick = LOWER(name_list[idx]);

    SELECT id FROM users WHERE nickname = nick INTO user_id2;
    RAISE NOTICE 'name_list[idx]: %, nick: %, user_id: %', name_list[idx], nick, user_id2;

    DELETE
    FROM chat_messages
    WHERE user_id = user_id2 AND (msg LIKE 'Demo message %' OR msg = '');

    idx := idx - 1;
  END LOOP;

  SELECT setval('chat_messages_id_seq', (SELECT COALESCE(MAX(id), 1) FROM chat_messages)) INTO idx;
  RAISE NOTICE 'chat_messages_id_seq: %', idx;
  SELECT setval('chat_message_logs_id_seq', (SELECT COALESCE(MAX(id), 1) FROM chat_message_logs)) INTO idx;
  RAISE NOTICE 'chat_message_logs_id_seq: %', idx;
END;
$$;

/*
 * Remove test data to the tables: users, streams.
 */
CALL remove_data_test_chat_messages();

DROP PROCEDURE IF EXISTS remove_data_test_chat_messages;

