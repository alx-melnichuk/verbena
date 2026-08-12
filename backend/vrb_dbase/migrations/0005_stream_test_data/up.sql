/*
 * Add test data
 */

CREATE OR REPLACE PROCEDURE add_user(
  _nickname VARCHAR, _email VARCHAR, _passwd VARCHAR, _user_id INOUT INTEGER
) LANGUAGE plpgsql
AS $$
BEGIN
  -- Add a new user.
  INSERT INTO users(nickname, email, "password", "role")
  VALUES(LOWER(_nickname), LOWER(_email), _passwd, 'user'::public."user_role")
  RETURNING id INTO _user_id;
END;
$$;


CREATE OR REPLACE PROCEDURE add_streams_by_interval(
  _user_id IN INTEGER,
  _count IN INTEGER,
  _user_nick IN VARCHAR,
  _trip IN VARCHAR,
  _starttime IN TIMESTAMPTZ,
  _add_interval IN VARCHAR
) LANGUAGE plpgsql
AS $$
DECLARE
  logo VARCHAR := '';
  idx_logo INTEGER := 0;
  idx INTEGER := 0;
  txt VARCHAR := '';
  title VARCHAR := '';
  year_str VARCHAR := '';
  stream_id INTEGER;
BEGIN
  idx_logo := 1;
  idx := 1;
  WHILE idx <= _count LOOP
    logo := CONCAT('/picture/trip_', _trip, '0', idx_logo, '.jpg');
    idx_logo := idx_logo + 1;
    IF idx_logo = 8 THEN
      idx_logo := 1;
    END IF;
    txt := CONCAT(UPPER(LEFT(SPLIT_PART(_user_nick,'_',1),1)), '.', INITCAP(SPLIT_PART(_user_nick,'_',2)));
    title := CONCAT('trip ', year_str, ' to ', _trip, ' ', idx, ' - ', txt);
    year_str := DATE_PART('year', _starttime);

    SELECT stream_id INTO stream_id
    FROM create_stream_and_stream_tag(
        _user_id,
        title,
        CONCAT('Description of a beautiful ', title), -- descript,
        logo,
        _starttime,
        NULL, -- _state stream_state,
        NULL, -- _started TIMESTAMPTZ,
        NULL, -- _paused TIMESTAMPTZ,
        NULL, -- _stopped TIMESTAMPTZ,
        NULL, -- _source VARCHAR,
        ARRAY['tourism', _trip] -- tags
    );

    RAISE NOTICE 'idx: %  create_stream_and_stream_tag(user_id: %, starttime: %) stream_id: %', idx, _user_id, _starttime, stream_id;

    _starttime := _starttime + _add_interval::interval;
    idx := idx + 1;
  END LOOP;
END;
$$;

CREATE OR REPLACE PROCEDURE add_data_test1()
LANGUAGE plpgsql 
AS $$
DECLARE
  user_index INTEGER := 0;
  user_id INTEGER := 0;
  name_list VARCHAR[];
  user_nick VARCHAR := '';
  trip_index INTEGER := 0;
  trip_list VARCHAR[];
  trip VARCHAR := '';
  starttime1 TIMESTAMPTZ;
  starttime2 TIMESTAMPTZ;
  starttime3 TIMESTAMPTZ;
BEGIN
  RAISE NOTICE 'Start';
  name_list := ARRAY[
    'Liam_Smith'  , 'Emma_Johnson' , 'Noah_Williams'  , 'Olivia_Jones',
    'Ethan_Brown' , 'Ava_Wilson'   , 'James_Miller'   , 'Mila_Davis'  ,
    'Logan_Lewis', 'Evelyn_Allen'
  ];

  trip_list := ARRAY['cyprus','france','greece','spain'];

  user_index := ARRAY_LENGTH(name_list, 1);
  WHILE user_index > 0 LOOP
    user_nick = LOWER(name_list[user_index]);
    RAISE NOTICE 'name_list[user_index]: %, user_nick: %', name_list[user_index], user_nick;

    DELETE FROM users WHERE nickname = user_nick;

    CALL add_user(
        user_nick,
        CONCAT(user_nick, '@gmail.us'),
        -- Pass_2
        '$argon2id$v=19$m=19456,t=2,p=1$eDqhmyjTHuR/AoCQjHD/oQ$EUG9u/tJesXpzJxLE5Y2JSDxirG4GF/7Alb6PlOrcLo',
        user_id
    );

    RAISE NOTICE 'user_nick: %, user_id: %', user_nick, user_id;
    
    starttime1 := current_date + make_time(cast(extract(timezone_hour from current_time) as integer) + 9, 0, 0);
    starttime1 := starttime1 + interval '1 day';
    
    starttime2 := current_date + make_time(cast(extract(timezone_hour from current_time) as integer) + 12, 0, 0);
    starttime2 := starttime2 + interval '1 day';

    starttime3 := current_date + make_time(cast(extract(timezone_hour from current_time) as integer) + 1, 0, 0);
    
    trip_index := ARRAY_LENGTH(trip_list, 1);
    WHILE trip_index > 0 LOOP
      
      trip := trip_list[trip_index];

      call add_streams_by_interval(user_id, 30, user_nick, trip, starttime1, '2 months');

    --   call add_streams_by_interval(user_id, 30, user_nick, trip, starttime2, '1 months');

      IF user_nick = 'logan_lewis' AND trip = 'france' THEN
        call add_streams_by_interval(user_id, 40, user_nick, trip, starttime3, '30 minute');
      END IF;

      IF user_nick = 'evelyn_allen' AND trip = 'cyprus' THEN
        call add_streams_by_interval(user_id, 40, user_nick, trip, starttime3, '30 minute');
      END IF;

      trip_index := trip_index - 1;
    END LOOP;

    user_index := user_index - 1;
  END LOOP;
END;
$$;


/*
 * Add test data to the tables: users, streams.
 */
CALL add_data_test1();


DROP PROCEDURE IF EXISTS add_data_test1;
DROP PROCEDURE IF EXISTS add_streams_by_interval;
DROP PROCEDURE IF EXISTS add_user;

