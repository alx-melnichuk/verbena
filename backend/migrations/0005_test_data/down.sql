/*
 * Remove test data.
 */
CREATE OR REPLACE PROCEDURE remove_data_test()
LANGUAGE plpgsql 
AS $$
DECLARE
  idx INTEGER := 0;
  name_list VARCHAR[];
  nick VARCHAR := '';
BEGIN
  RAISE NOTICE 'Start';
  name_list := ARRAY[
    'Liam_Smith'  , 'Emma_Johnson' , 'Noah_Williams'  , 'Olivia_Jones' ,
    'Ethan_Brown' , 'Ava_Wilson'   , 'James_Miller'   , 'Mila_Davis'   ,
    'Jack_Thomas' , 'Sophia_Taylor', 'Jacob_Moore'    , 'Emily_White'  ,
    'John_Harris' , 'Mia_Jackson'  , 'Lucas_Anderson' , 'Amelia_Martin',
    'Mason_Garcia', 'Harper_Clark' , 'Logan_Lewis'    , 'Evelyn_Allen'
  ];
   
  idx := ARRAY_LENGTH(name_list, 1);
  WHILE idx > 0 LOOP
    nick = LOWER(name_list[idx]);
    RAISE NOTICE 'name_list[idx]: %, nick: %', name_list[idx], nick;

    DELETE FROM users WHERE nickname = nick;

    idx := idx - 1;
  END LOOP;

  SELECT setval('users_id_seq', (SELECT COALESCE(MAX(id), 1) FROM users)) INTO idx;
  RAISE NOTICE 'users_id_seq: %', idx;
  SELECT setval('streams_id_seq', (SELECT COALESCE(MAX(id), 1) FROM streams)) INTO idx;
  RAISE NOTICE 'streams_id_seq: %', idx;
  SELECT setval('stream_tags_id_seq', (SELECT COALESCE(MAX(id), 1) FROM stream_tags)) INTO idx;
  RAISE NOTICE 'stream_tags_id_seq: %', idx;

END;
$$;

/*
 * Remove test data to the tables: users, streams.
 */
CALL remove_data_test();

DROP PROCEDURE IF EXISTS remove_data_test;

