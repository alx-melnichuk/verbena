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
END;
$$;

CREATE OR REPLACE PROCEDURE add_stream(
  user_id1 INTEGER, title1 VARCHAR, logo1 VARCHAR, 
  starttime TIMESTAMP WITH TIME ZONE, descript VARCHAR,
  stream_id INOUT INTEGER
) LANGUAGE plpgsql
AS $$
BEGIN
  INSERT INTO streams(user_id, title, logo, starttime, descript)
  VALUES(user_id1, title1, logo1, starttime, descript)
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


CREATE OR REPLACE PROCEDURE add_data_test1()
LANGUAGE plpgsql 
AS $$
DECLARE
  user_index INTEGER := 0;
  user_id INTEGER := 0;
  name_list VARCHAR[];
  nick VARCHAR := '';
  title VARCHAR := '';
  descript VARCHAR := '';
  stream_id INTEGER := 0;
  stream_tag_id INTEGER := 0;
  tourism_tag_id INTEGER := 0;
  trip_index INTEGER := 0;
  index_day INTEGER := 0;
  trip_list VARCHAR[];
  trip VARCHAR := '';
  logo VARCHAR := '';
  tag_name VARCHAR := '';
  idx INTEGER := 0;
  txt VARCHAR := '';
  year_str VARCHAR := '';
  starttime1 TIMESTAMP WITH TIME ZONE;
  starttime2 TIMESTAMP WITH TIME ZONE;
BEGIN
  RAISE NOTICE 'Start';
  name_list := ARRAY[
    'Liam_Smith'  , 'Emma_Johnson' , 'Noah_Williams'  , 'Olivia_Jones',
    'Ethan_Brown' , 'Ava_Wilson'   , 'James_Miller'   , 'Mila_Davis'
  ];

  trip_list := ARRAY['cyprus','france','greece','spain'];
   
  user_index := ARRAY_LENGTH(name_list, 1);
  index_day := user_index;
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

    starttime1:= '2026-03-10T10:00:00+02';
    starttime2:= '2026-02-02T10:00:00+02';

    trip_index := ARRAY_LENGTH(trip_list, 1);
    WHILE trip_index > 0 LOOP
      
      trip := trip_list[trip_index];

      CALL add_stream_tag(user_id, trip, stream_tag_id);

      idx := 1;
      WHILE idx <= 7 LOOP
        logo := CONCAT('/assets/images/trip_', trip, '0', idx, '.jpg');
        txt := CONCAT(UPPER(LEFT(SPLIT_PART(nick,'_',1),1)), '.', INITCAP(SPLIT_PART(nick,'_',2)));

        year_str := DATE_PART('year', starttime1);
        title := CONCAT('trip ', year_str, ' to ', trip, ' ', idx, ' - ', txt);
        descript := CONCAT('Description of a beautiful ', title);

        CALL add_stream(user_id, title, logo, starttime1, descript, stream_id);
        CALL add_link_stream_tags_to_streams(tourism_tag_id, stream_id);
        CALL add_link_stream_tags_to_streams(stream_tag_id, stream_id);

        RAISE NOTICE 'idx: %  CALL add_stream(user_id: %) stream_id: %', idx, user_id, stream_id;
        starttime1 := starttime1 + interval '4 months'; -- '1 years';

        IF user_index = index_day THEN
          year_str := DATE_PART('year', starttime2);
          title := CONCAT('trip ', year_str, ' to ', trip, ' ', idx, ' - ', txt);
          descript := CONCAT('Description of a beautiful ', title);

          CALL add_stream(user_id, title, logo, starttime2, descript, stream_id);
          CALL add_link_stream_tags_to_streams(tourism_tag_id, stream_id);
          CALL add_link_stream_tags_to_streams(stream_tag_id, stream_id);

          RAISE NOTICE 'idx: %  CALL add_stream(user_id: %) stream_id: %', idx, user_id, stream_id;
          starttime2 := starttime2 + interval '30 minute';
        END IF;

        idx := idx + 1;
      END LOOP;

      trip_index := trip_index - 1;
    END LOOP;

    user_index := user_index - 1;
  END LOOP;
END;
$$;


CREATE OR REPLACE PROCEDURE add_data_test2()
LANGUAGE plpgsql 
AS $$
DECLARE
  user_index INTEGER := 0;
  user_id INTEGER := 0;
  name_list VARCHAR[];
  nick VARCHAR := '';
  title VARCHAR := '';
  descript VARCHAR := '';
  stream_id INTEGER := 0;
  stream_tag_id INTEGER := 0;
  tourism_tag_id INTEGER := 0;
  trip_index INTEGER := 0;
  index_day INTEGER := 0;
  trip_list VARCHAR[];
  trip_id_list INTEGER[];
  trip VARCHAR := '';
  logo VARCHAR := '';
  tag_name VARCHAR := '';
  idx INTEGER := 0;
  txt VARCHAR := '';
  year_str VARCHAR := '';
  year_idx INTEGER := 0;
  startdate1 TIMESTAMP WITH TIME ZONE;
BEGIN
  RAISE NOTICE 'Start';
  name_list := ARRAY['Logan_Lewis', 'Evelyn_Allen'];

  user_index := ARRAY_LENGTH(name_list, 1);
  WHILE user_index > 0 LOOP
    nick = LOWER(name_list[user_index]);
    DELETE FROM users WHERE nickname = nick;
    user_index := user_index - 1;
  END LOOP;

  -- Create a trip list. There are 7 photos for each type.
  trip_list := ARRAY['cyprus','france','greece','spain'];

  user_index := ARRAY_LENGTH(name_list, 1);
  index_day := user_index;
  WHILE user_index > 0 LOOP
    nick = LOWER(name_list[user_index]);
    RAISE NOTICE 'name_list[user_index]: %, nick: %', name_list[user_index], nick;
    
    -- Delete the previous version of the data.
    DELETE FROM users WHERE nickname = nick;
    
    -- Create a new user with the specified nickname.
    CALL add_user(
        nick,
        CONCAT(nick, '@gmail.us'),
        -- Pass_2
        '$argon2id$v=19$m=19456,t=2,p=1$eDqhmyjTHuR/AoCQjHD/oQ$EUG9u/tJesXpzJxLE5Y2JSDxirG4GF/7Alb6PlOrcLo',
        user_id
    );
    
    -- Create an "tourism" tag for a new user and get his ID.
    CALL add_stream_tag(user_id, 'tourism', tourism_tag_id);

    trip_id_list := ARRAY[]::INTEGER[];
    -- For each element in the trips array.
    trip_index := ARRAY_LENGTH(trip_list, 1);
    idx := 1;
    WHILE idx <= trip_index LOOP
      trip := trip_list[idx];
      -- Create an "name_trip" tag for a new user and get his ID.
      CALL add_stream_tag(user_id, trip, stream_tag_id);
      -- Add the new tag ID to the ID array.
      trip_id_list := ARRAY_APPEND(trip_id_list, stream_tag_id);

      idx := idx + 1;
    END LOOP;

    year_idx := 2026;
    WHILE year_idx < 2037 LOOP
      startdate1 := to_timestamp(CONCAT(year_idx,'/01/01 08:00:00'), 'YYYY/MM/DD HH24:MI:SS');

      -- For each element in the trips array.
      trip_index := ARRAY_LENGTH(trip_list, 1);
      WHILE trip_index > 0 LOOP
        -- Get the name of the tag with the "trip_index" index.
        trip := trip_list[trip_index];
        -- Get the ID of the tag with the "trip_index" index.
        stream_tag_id := trip_id_list[trip_index];

        IF trip_index = 2 THEN
          startdate1 := to_timestamp(CONCAT(year_idx,'/07/01 08:00:00'), 'YYYY/MM/DD HH24:MI:SS');
        END IF;

        idx := 1;
        WHILE idx <= 7 LOOP
          logo := CONCAT('/assets/images/trip_', trip, '0', idx, '.jpg');
          txt := CONCAT(UPPER(LEFT(SPLIT_PART(nick,'_',1),1)), '.', INITCAP(SPLIT_PART(nick,'_',2)));

          year_str := DATE_PART('year', startdate1);
          title := CONCAT('trip ', year_str, ' to ', trip, ' ', idx, ' - ', txt);
          descript := CONCAT('Description of a beautiful ', title);
          -- Create a stream for a user and return his ID. 
          CALL add_stream(user_id, title, logo, startdate1, descript, stream_id);
          -- Add an "tourism" tag for a new stream.
          CALL add_link_stream_tags_to_streams(tourism_tag_id, stream_id);
          -- Add an "name_trip" tag for a new stream.
          CALL add_link_stream_tags_to_streams(stream_tag_id, stream_id);

          startdate1 := startdate1 + interval '2 days';

          IF idx = 3 OR idx = 5 OR idx = 7 THEN
            startdate1 := startdate1 + interval '1 months';
          END IF;

          idx := idx + 1;
        END LOOP;

        trip_index := trip_index - 1;
      END LOOP;

      year_idx := year_idx + 1;
    END LOOP;

    user_index := user_index - 1;
  END LOOP;
END;
$$;

/*
 * Add test data to the tables: users, streams.
 */
CALL add_data_test1();
CALL add_data_test2();


DROP PROCEDURE IF EXISTS add_data_test1;
DROP PROCEDURE IF EXISTS add_data_test2;
DROP PROCEDURE IF EXISTS add_user;
DROP PROCEDURE IF EXISTS add_stream;
DROP PROCEDURE IF EXISTS add_stream_tag;
DROP PROCEDURE IF EXISTS add_link_stream_tags_to_streams;

