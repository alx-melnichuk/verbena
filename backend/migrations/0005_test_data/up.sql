/*
 * Add test data
 */

CREATE OR REPLACE PROCEDURE add_user(
  nickname1 VARCHAR, email1 VARCHAR, passwd1 VARCHAR, user_id1 INOUT INTEGER
) LANGUAGE plpgsql
AS $$
BEGIN
  -- Add a new user.
  INSERT INTO users(nickname, email, "password", "role")
  VALUES(LOWER(nickname1), LOWER(email1), passwd1, 'user'::public."user_role")
  RETURNING id INTO user_id1;
  -- Add a session for a new user.
  INSERT INTO sessions(user_id)
  VALUES(user_id1);
END;
$$;

CREATE OR REPLACE PROCEDURE add_stream(
  user_id1 INTEGER, title1 VARCHAR, logo1 VARCHAR, num_days INTEGER, stream_id INOUT INTEGER
) LANGUAGE plpgsql
AS $$
BEGIN
  INSERT INTO streams(user_id, title, logo, starttime)
  VALUES(user_id1, title1, logo1, now() + interval '1 day' * num_days)
  RETURNING id INTO stream_id;
END;
$$;

CREATE OR REPLACE PROCEDURE add_stream_tag(
  user_id1 INTEGER, tag_name VARCHAR, stream_tag_id INOUT INTEGER
) LANGUAGE plpgsql
AS $$
BEGIN
  INSERT INTO stream_tags(user_id, "name")
  VALUES(user_id1, tag_name)
  RETURNING id INTO stream_tag_id;
END;
$$;

CREATE OR REPLACE PROCEDURE add_link_stream_tags_to_streams(
  stream_tag_id1 INTEGER, stream_id1 INTEGER
) LANGUAGE plpgsql
AS $$
BEGIN
  INSERT INTO link_stream_tags_to_streams(stream_tag_id, stream_id)
  VALUES(stream_tag_id1, stream_id1);
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
  stream_id INTEGER := 0;
  stream_tag_id INTEGER := 0;
  tourism_tag_id INTEGER := 0;
  trip_index INTEGER := 0;
  trip_list VARCHAR[];
  trip VARCHAR := '';
  logo VARCHAR := '';
  tag_name VARCHAR := '';
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
    
    CALL add_stream_tag(user_id, 'tourism', tourism_tag_id);

    trip_index := ARRAY_LENGTH(trip_list, 1);
    WHILE trip_index > 0 LOOP
      
      trip := trip_list[trip_index];

      CALL add_stream_tag(user_id, trip, stream_tag_id);

      idx := 1;
      WHILE idx <= 7 LOOP
        logo := CONCAT('/assets/images/trip_', trip, '0', idx, '.jpg');
        title := CONCAT(UPPER(LEFT(SPLIT_PART(nick,'_',1),1)), '.', INITCAP(SPLIT_PART(nick,'_',2)));
        CALL add_stream(user_id, CONCAT('trip to ', trip, ' ', idx, ' - ', title), logo, idx + 2, stream_id);

        RAISE NOTICE 'idx: %  CALL add_stream(user_id: %) stream_id: %', idx, user_id, stream_id;

        CALL add_link_stream_tags_to_streams(tourism_tag_id, stream_id);

        CALL add_link_stream_tags_to_streams(stream_tag_id, stream_id);

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
DROP PROCEDURE IF EXISTS add_stream_tag;
DROP PROCEDURE IF EXISTS add_link_stream_tags_to_streams;

