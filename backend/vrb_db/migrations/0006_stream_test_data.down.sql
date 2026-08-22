/*
 * Remove 10 test users and test streams for them.
 * Set the current values for the ID generators.
 */
CREATE OR REPLACE PROCEDURE remove_data_test()
LANGUAGE plpgsql 
AS $$
DECLARE
  idx INTEGER := 0;
  name_list VARCHAR[];
  user_nick VARCHAR := '';
BEGIN
  RAISE NOTICE 'Start';
  name_list := ARRAY[
    'Liam_Smith'  , 'Emma_Johnson' , 'Noah_Williams'  , 'Olivia_Jones',
    'Ethan_Brown' , 'Ava_Wilson'   , 'James_Miller'   , 'Mila_Davis',
    'Logan_Lewis', 'Evelyn_Allen'
  ];
   
  idx := ARRAY_LENGTH(name_list, 1);
  WHILE idx > 0 LOOP
    user_nick = LOWER(name_list[idx]);
    RAISE NOTICE 'name_list[idx]: %, user_nick: %', name_list[idx], user_nick;

    DELETE FROM users WHERE nickname = user_nick;

    idx := idx - 1;
  END LOOP;

  SELECT setval('users_id_seq', (SELECT COALESCE(MAX(id), 1) FROM users)) INTO idx;
  RAISE NOTICE 'users_id_seq: %', idx;

  SELECT setval('streams_id_seq', (SELECT COALESCE(MAX(id), 1) FROM streams)) INTO idx;
  RAISE NOTICE 'streams_id_seq: %', idx;

  SELECT setval('stream_tags_id_seq', (SELECT COALESCE(MAX(id), 1) FROM stream_tags)) INTO idx;
  RAISE NOTICE 'stream_tags_id_seq: %', idx;

  SELECT setval('link_stream_tags_to_streams_id_seq', (SELECT COALESCE(MAX(id), 1) FROM link_stream_tags_to_streams)) INTO idx;
  RAISE NOTICE 'link_stream_tags_to_streams_id_seq: %', idx;
END;
$$;

/*
 * Remove test data to the tables: users, streams.
 */
CALL remove_data_test();

DROP PROCEDURE IF EXISTS remove_data_test;

