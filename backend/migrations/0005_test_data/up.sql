/*
 * Add test data
 */

CREATE OR REPLACE PROCEDURE add_user(nickname1 VARCHAR, email1 VARCHAR, passwd1 VARCHAR, user_id INOUT INTEGER) 
LANGUAGE plpgsql
AS $$
BEGIN
  INSERT INTO users(nickname, email, "password", "role")
  VALUES(LOWER(nickname1), LOWER(email1), passwd1, 'user'::public."user_role")
  RETURNING id INTO user_id;
END;
$$;

CREATE OR REPLACE PROCEDURE add_stream(user_id1 INTEGER, title1 VARCHAR, logo1 VARCHAR, num_days INTEGER) 
LANGUAGE plpgsql
AS $$
BEGIN
  INSERT INTO streams(user_id, title, logo, starttime)
  VALUES(user_id1, title1, logo1, now() + interval '1 day' * num_days);
END;
$$;


CREATE OR REPLACE PROCEDURE add_data_test()
LANGUAGE plpgsql 
AS $$
DECLARE
  user_index INTEGER := 0;
  user_id INTEGER := 0;
  name_list VARCHAR[];
  nick VARCHAR := '';
  title VARCHAR := '';
  trip_index INTEGER := 0;
  trip_list VARCHAR[];
  trip VARCHAR := '';
  logo VARCHAR := '';
  idx INTEGER := 0;
BEGIN
  RAISE NOTICE 'Start';
  name_list := ARRAY[
    'Liam_Smith'  , 'Emma_Johnson' , 'Noah_Williams'  , 'Olivia_Jones' ,
    'Ethan_Brown' , 'Ava_Wilson'   , 'James_Miller'   , 'Mila_Davis'   ,
    'Jack_Thomas' , 'Sophia_Taylor', 'Jacob_Moore'    , 'Emily_White'  ,
    'John_Harris' , 'Mia_Jackson'  , 'Lucas_Anderson' , 'Amelia_Martin',
    'Mason_Garcia', 'Harper_Clark' , 'Logan_Lewis'    , 'Evelyn_Allen'
  ];

  trip_list := ARRAY['cyprus','france','greece','spain'];
   
  user_index := ARRAY_LENGTH(name_list, 1);
  WHILE user_index > 0 LOOP
    nick = LOWER(name_list[user_index]);
    RAISE NOTICE 'name_list[user_index]: %, nick: %', name_list[user_index], nick;

    DELETE FROM users WHERE nickname = nick;

    CALL add_user(
        nick,
        CONCAT(nick, '@gmail.us'),
        -- Pass_2
        '$argon2id$v=19$m=19456,t=2,p=1$eDqhmyjTHuR/AoCQjHD/oQ$EUG9u/tJesXpzJxLE5Y2JSDxirG4GF/7Alb6PlOrcLo',
        user_id
    );

    RAISE NOTICE 'nick: %, user_id: %', nick, user_id;
    
    trip_index := ARRAY_LENGTH(trip_list, 1);
    WHILE trip_index > 0 LOOP
      
      trip := trip_list[trip_index];
      idx := 1;
      WHILE idx <= 7 LOOP
        logo := CONCAT('/assets/images/trip_', trip, '0', idx, '.jpg');
        title := CONCAT(UPPER(LEFT(SPLIT_PART(nick,'_',1),1)), '.', INITCAP(SPLIT_PART(nick,'_',2)));
        CALL add_stream(user_id, CONCAT('trip to ', trip, ' ', idx, ' - ', title), logo, idx + 2);
        RAISE NOTICE 'idx: %  CALL add_stream(user_id: %)', idx, user_id;
        idx := idx + 1;
      END LOOP;

      trip_index := trip_index - 1;
    END LOOP;
    
    user_index := user_index - 1;
  END LOOP;
END;
$$;


/*
 * Add test data to the tables: users, streams.
 */
CALL add_data_test();


DROP PROCEDURE IF EXISTS add_data_test;
DROP PROCEDURE IF EXISTS add_user;
DROP PROCEDURE IF EXISTS add_stream;

