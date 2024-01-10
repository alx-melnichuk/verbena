/*
 * Remove test data.
 */
CREATE OR REPLACE PROCEDURE remove_data_test()
LANGUAGE plpgsql 
AS $$
DECLARE
  user_index INTEGER := 0;
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
   
  user_index := ARRAY_LENGTH(name_list, 1);
  WHILE user_index > 0 LOOP
    nick = LOWER(name_list[user_index]);
    RAISE NOTICE 'name_list[user_index]: %, nick: %', name_list[user_index], nick;

    DELETE FROM users WHERE nickname = nick;

    user_index := user_index - 1;
  END LOOP;

  SELECT setval('users_id_seq', (SELECT MAX(id) FROM users)) INTO user_index;
END;
$$;

/*
 * Remove test data to the tables: users, streams.
 */
CALL remove_data_test();

DROP PROCEDURE IF EXISTS remove_data_test;

