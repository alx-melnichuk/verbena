
/* Create a new "stream_state" type. */
CREATE TYPE stream_state AS ENUM ('waiting', 'started', 'stopped', 'paused', 'preparing');

/* Create table "streams". */
CREATE TABLE streams (
    id SERIAL PRIMARY KEY NOT NULL,
    /* Owner id */
    user_id INT REFERENCES users(id) ON DELETE CASCADE,
    /* Custom title */
    title VARCHAR(255) NOT NULL,
    /* Custom description */
    descript TEXT DEFAULT '',
    /* Link to stream logo, optional */
    logo VARCHAR(255) NULL,
    /* stream source */
    source VARCHAR(255) DEFAULT 'obs',
    /* Time when stream should start. Required on create */
    starttime TIMESTAMP WITH TIME ZONE NOT NULL,
    /* Stream live status, false means inactive */
    live BOOLEAN DEFAULT FALSE,
    /* Stream live state - waiting, preparing, start, paused, stop (waiting by default) */
    "state" stream_state NOT NULL DEFAULT 'waiting',
    /* Time when stream was started */
    "started" TIMESTAMP WITH TIME ZONE NULL,
    /* Time when stream was stopped */
    "stopped" TIMESTAMP WITH TIME ZONE NULL,
    /* Stream status, false means disabled */
    "status" BOOLEAN DEFAULT TRUE
);

-- CREATE INDEX idx_streams_final_date_nickname ON streams(final_date, nickname);
-- CREATE INDEX idx_streams_final_date_email ON streams(final_date, email);
-- CREATE INDEX idx_streams_final_date ON streams(final_date);




-- DROP PROCEDURE add_user;
CREATE OR REPLACE PROCEDURE add_user(nickname VARCHAR, email VARCHAR, passwd VARCHAR, user_id INOUT INTEGER
) language plpgsql AS $$
BEGIN
  INSERT INTO users(nickname, email, "password", "role")
  VALUES(nickname, email, passwd, 'user'::public."user_role")
  RETURNING id INTO user_id;
END;$$

-- DROP PROCEDURE add_user;
CREATE OR REPLACE PROCEDURE add_stream(user_id INTEGER, title VARCHAR, logo VARCHAR, num_days INTEGER
) language plpgsql AS $$
BEGIN
  INSERT INTO streams(user_id, title, logo, starttime)
  VALUES(user_id, title, logo, now() + interval '1 day' * num_days);
END;$$

-- DROP PROCEDURE add_user;
-- DO $$
CREATE OR REPLACE PROCEDURE add_data_test() language plpgsql AS $$
DECLARE
  user_index INTEGER := 0;
  user_id INTEGER := 0;
  name_list VARCHAR[];
  nick VARCHAR := '';
  trip_index INTEGER := 0;
  trip_list VARCHAR[];
  trip VARCHAR := '';
  logo VARCHAR := '';
  idx INTEGER := 0;
BEGIN
  name_list := ARRAY['John_Smith', 'Lisa_Brown', 'Amy_Jones'];
  trip_list := ARRAY['cyprus','france','greece','spain'];

  user_index := 1;
  WHILE user_index <= 3 LOOP
    nick = name_list[user_index];

    DELETE FROM users WHERE nickname = nick;

    CALL add_user(
        nick,
        CONCAT(nick, '@gmail.us'),
        '$argon2id$v=19$m=19456,t=2,p=1$eDqhmyjTHuR/AoCQjHD/oQ$EUG9u/tJesXpzJxLE5Y2JSDxirG4GF/7Alb6PlOrcLo',
        user_id
    );

    RAISE NOTICE 'nick: %, user_id: %', nick, user_id;

    trip_index := 1;
    WHILE trip_index <= 4 LOOP

      trip := trip_list[trip_index];
      idx := 1;
      WHILE idx <= 7 LOOP
        logo := CONCAT('/assets/images/trip_', trip, '0', idx, '.jpg');
        CALL add_stream(user_id, CONCAT('trip to ', trip, ' ', idx, ' - ', nick), logo, idx + 2);
        RAISE NOTICE 'idx: %  CALL add_stream(user_id: %)', idx, user_id;
        idx := idx + 1;
      END LOOP;

      COMMIT;
      trip_index := trip_index + 1;
    END LOOP;

    user_index := user_index + 1;    
  END LOOP;
END;$$
-- END $$

-- CALL add_data_test();


