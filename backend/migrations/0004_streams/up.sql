
/* Create a type "stream_state". */
CREATE TYPE stream_state AS ENUM ('waiting', 'started', 'stopped', 'paused', 'preparing');

/* Create "streams" table. */
CREATE TABLE streams (
    id SERIAL PRIMARY KEY NOT NULL,
    /* Owner id */
    user_id INT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    /* Custom title */
    title VARCHAR(255) NOT NULL,
    /* Custom description */
    descript TEXT NOT NULL DEFAULT '',
    /* Link to stream logo, optional */
    logo VARCHAR(255) NULL,
    /* Time when stream should start. Required on create */
    starttime TIMESTAMP WITH TIME ZONE NOT NULL,
    /* Stream live status, false means inactive */
    live BOOLEAN NOT NULL DEFAULT FALSE,
    /* Stream live state - waiting, preparing, start, paused, stop (waiting by default) */
    "state" stream_state NOT NULL DEFAULT 'waiting',
    /* Time when stream was started */
    "started" TIMESTAMP WITH TIME ZONE NULL,
    /* Time when stream was stopped */
    "stopped" TIMESTAMP WITH TIME ZONE NULL,
    /* Stream status, false means disabled */
    "status" BOOLEAN NOT NULL DEFAULT TRUE,
    /* stream source */
    source VARCHAR(255) NOT NULL DEFAULT 'obs',
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);

SELECT diesel_manage_updated_at('streams');

CREATE INDEX idx_streams_user_id ON streams(user_id);
CREATE INDEX idx_streams_live ON streams(live);
CREATE INDEX idx_streams_status ON streams("status");


/* Create "stream_tags" table. */
CREATE TABLE stream_tags (
    id SERIAL PRIMARY KEY NOT NULL,
    /* Owner id */
    user_id INT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    /* Custom tag name */
    "name" VARCHAR(255) NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT CURRENT_TIMESTAMP
);

SELECT diesel_manage_updated_at('stream_tags');

CREATE UNIQUE INDEX uq_idx_stream_tags_user_id_name ON stream_tags(user_id, "name");
CREATE INDEX idx_stream_tags_user_id ON stream_tags(user_id);
CREATE INDEX idx_stream_tags_name ON stream_tags("name");


/* Create "link_stream_tags_to_streams" table. */

CREATE TABLE link_stream_tags_to_streams (
    id SERIAL PRIMARY KEY NOT NULL,
    /* Stream Tag id */
    stream_tag_id INT NOT NULL REFERENCES stream_tags(id),
    /* Stream id */
    stream_id INT NOT NULL REFERENCES streams(id) ON DELETE CASCADE
);

CREATE INDEX idx_link_stream_tags_to_streams_stream_id_stream_tag_id ON link_stream_tags_to_streams(stream_id, stream_tag_id);


/*
 * Stored procedures for working with data from the "stream_tags" table.
 */

/* Add a link to "tag" for "stream". */
CREATE OR REPLACE PROCEDURE add_stream_tag_to_stream(
  user_id1 INTEGER, tag_name1 VARCHAR, stream_id1 INTEGER
) LANGUAGE plpgsql
AS $$
DECLARE
  stream_tag_id1 INTEGER;
  link_tag_stream_id1 INTEGER;
BEGIN
  -- Find for "stream tag" with the value "name tag".
  SELECT id INTO stream_tag_id1
  FROM stream_tags
  WHERE user_id = user_id1 AND "name" = tag_name1;

  -- If "stream tag" is not found, then add it.
  IF NOT FOUND THEN
    INSERT INTO stream_tags(user_id, "name")
    VALUES(user_id1, tag_name1)
    RETURNING id INTO stream_tag_id1;
  ELSE
    -- Find "link_stream_tags_to_streams" with the given: stream_id1 and stream_tag_id1.
    SELECT id INTO link_tag_stream_id1
    FROM link_stream_tags_to_streams
    WHERE stream_id = stream_id1 AND stream_tag_id = stream_tag_id1;
  END IF;

  -- If "link_stream_tags_to_streams" is not found, then add it.
  IF (link_tag_stream_id1 IS NULL) THEN
    INSERT INTO link_stream_tags_to_streams(stream_tag_id, stream_id)
    VALUES(stream_tag_id1, stream_id1)
    RETURNING id INTO link_tag_stream_id1;
  END IF;
  -- raise notice 'stream_tag_id1: %, link_tag_stream_id1: %', stream_tag_id1, link_tag_stream_id1;
END;
$$;

/* Add links to the "tag" list for "stream". */
CREATE OR REPLACE PROCEDURE add_list_stream_tag_to_stream(
  user_id1 INTEGER, stream_id1 INTEGER, tag_name_list1 VARCHAR
) LANGUAGE plpgsql
AS $$
DECLARE
  tag_index INTEGER;
  tag_name_buf VARCHAR[];
  tag_name VARCHAR;
BEGIN
  tag_name_buf := string_to_array(tag_name_list1, ',');
  tag_index := ARRAY_LENGTH(tag_name_buf, 1);
  WHILE tag_index > 0 LOOP
    tag_name = TRIM(tag_name_buf[tag_index]);
    -- raise notice 'tag_index: %, tag_name: %', tag_index, tag_name;
    CALL add_stream_tag_to_stream(user_id1, tag_name, stream_id1);
    tag_index := tag_index - 1;
  END LOOP;
END;
$$;

/* Remove link to "tag" for "stream". */
CREATE OR REPLACE PROCEDURE remove_stream_tag_to_stream(
  user_id1 INTEGER, tag_name1 VARCHAR, stream_id1 INTEGER
) LANGUAGE plpgsql
AS $$
DECLARE
  stream_tag_id1 INTEGER;
  link_tag_stream_id1 INTEGER;
  count_id INTEGER;
  exist_val INTEGER;
BEGIN
  -- Find for "stream tag" with the value "name tag".
  SELECT id INTO stream_tag_id1
  FROM stream_tags
  WHERE user_id = user_id1 AND "name" = tag_name1;
  raise notice 'A: stream_tag_id1: %', stream_tag_id1;

  -- If "stream tag" is not found, then exit.
  IF (stream_tag_id1 IS NULL) THEN
    RETURN;
  END IF;
  
  -- Find "link_stream_tags_to_streams" with the given: stream_id1 and stream_tag_id1.
  SELECT id INTO link_tag_stream_id1
  FROM link_stream_tags_to_streams
  WHERE stream_tag_id = stream_tag_id1 AND stream_id = stream_id1;
  raise notice 'B: link_tag_stream_id1: %', link_tag_stream_id1;

  -- If "link_stream_tags_to_streams" is found, then remove it.
  IF (link_tag_stream_id1 IS NOT NULL) THEN
    DELETE
    FROM link_stream_tags_to_streams
    WHERE id = link_tag_stream_id1;
    raise notice 'C: delete link_stream link_tag_stream_id1: %', link_tag_stream_id1;
  END IF;

  -- If there are no more entries for stream_tag_id in the link table, then delete the tag itself.
  IF NOT EXISTS(SELECT 1 FROM link_stream_tags_to_streams WHERE stream_tag_id = stream_tag_id1 LIMIT 1) THEN
    DELETE
    FROM stream_tags
    WHERE id = stream_tag_id1;
    raise notice 'D: delete stream_tags stream_tag_id1: %', stream_tag_id1;
  END IF;
END;
$$;

/* Remove links to the "tag" list for "stream". */
CREATE OR REPLACE PROCEDURE remove_list_stream_tag_to_stream(
  user_id1 INTEGER, stream_id1 INTEGER, tag_name_list1 VARCHAR
) LANGUAGE plpgsql
AS $$
DECLARE
  tag_index INTEGER := 0;
  tag_name_buf VARCHAR[];
  tag_name VARCHAR := '';
BEGIN
  tag_name_buf := string_to_array(tag_name_list1, ',');
  tag_index := ARRAY_LENGTH(tag_name_buf, 1);
  WHILE tag_index > 0 LOOP
    tag_name = TRIM(tag_name_buf[tag_index]);
    -- raise notice 'tag_index: %, tag_name: %', tag_index, tag_name;
    CALL remove_stream_tag_to_stream(user_id1, tag_name, stream_id1);
    tag_index := tag_index - 1;
  END LOOP;
END;
$$;

/* The function returns the "stream_tags" data for the specified "user" and "stream". */
CREATE OR REPLACE FUNCTION get_stream_tags_by_streams(
  user_id1 INTEGER, stream_id1 INTEGER    
) RETURNS SETOF stream_tags LANGUAGE sql
AS $$
  -- Get the "stream_tags" data for the specified "user" and "stream".
  SELECT 
    T.*
  FROM
    stream_tags T,
    link_stream_tags_to_streams L
  WHERE
    T.user_id = user_id1
    AND T.id = L.stream_tag_id
    AND L.stream_id = stream_id1;
$$;

-- select * from get_stream_tags_by_streams(182, 562);

