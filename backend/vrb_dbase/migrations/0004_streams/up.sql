
/* Create a type "stream_state".
  Accepts the following values:
    waiting - stream is waiting (default),
    preparing - stream is preparing (is live),
    started - stream has started (is live),
    paused - stream is paused (is live),
    stopped - stream is stopped
 */
CREATE TYPE stream_state AS ENUM ('waiting', 'preparing', 'started', 'paused', 'stopped');

/* Create "streams" table. */
CREATE TABLE streams (
    id SERIAL PRIMARY KEY NOT NULL,
    /* Owner id */
    user_id INT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    /* Custom title */
    title VARCHAR(255) NOT NULL,
    /* Custom description */
    descript TEXT DEFAULT '' NOT NULL,
    /* Link to stream logo, optional */
    logo VARCHAR(255) NULL,
    /* Time when stream should start. Required on create */
    starttime TIMESTAMP WITH TIME ZONE NOT NULL,
    /* Stream live status, false means inactive */
    live BOOLEAN NOT NULL DEFAULT FALSE,
    /* Stream live state - waiting (default), preparing, start, paused, stopped. */
    "state" stream_state DEFAULT 'waiting' NOT NULL,
    /* Time when stream was started */
    "started" TIMESTAMP WITH TIME ZONE NULL,
    /* Time when stream was stopped */
    "stopped" TIMESTAMP WITH TIME ZONE NULL,
    /* stream source */
    source VARCHAR(255) DEFAULT 'obs' NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP NOT NULL
);

SELECT diesel_manage_updated_at('streams');

CREATE INDEX idx_streams_user_id ON streams(user_id);
CREATE INDEX idx_streams_starttime ON streams(starttime);
CREATE INDEX idx_streams_live ON streams(live);
CREATE INDEX idx_streams_state ON streams("state");


/* Create trigger function for table "streams". */
CREATE OR REPLACE FUNCTION modify_stream_set_live()
RETURNS TRIGGER 
LANGUAGE plpgsql 
AS $$
BEGIN
  NEW.live := NEW."state" IN ('preparing', 'started', 'paused');
  IF NEW."state" = 'started' THEN
    NEW.started := CURRENT_TIMESTAMP;
  END IF;
  IF NEW."state" = 'stopped' THEN
    NEW.stopped := CURRENT_TIMESTAMP;
  END IF;
  RETURN NEW;
END;
$$;

/* Create trigger for table "streams". */
CREATE OR REPLACE TRIGGER tr_before_insert_stream_set_live
BEFORE INSERT ON streams
FOR EACH ROW
EXECUTE FUNCTION modify_stream_set_live();

CREATE OR REPLACE TRIGGER tr_before_update_stream_set_live
BEFORE UPDATE ON streams
FOR EACH ROW
EXECUTE FUNCTION modify_stream_set_live();



/* Create "stream_tags" table. */
CREATE TABLE stream_tags (
    id SERIAL PRIMARY KEY NOT NULL,
    /* Owner id */
    user_id INT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    /* Custom tag name */
    "name" VARCHAR(255) NOT NULL
);

CREATE UNIQUE INDEX uq_idx_stream_tags_user_id_name ON stream_tags(user_id, "name");



/* Create "link_stream_tags_to_streams" table. */

CREATE TABLE link_stream_tags_to_streams (
    id SERIAL PRIMARY KEY NOT NULL,
    /* Stream Tag id */
    stream_tag_id INT NOT NULL REFERENCES stream_tags(id),
    /* Stream id */
    stream_id INT NOT NULL REFERENCES streams(id) ON DELETE CASCADE
);

CREATE INDEX idx_link_stream_tags_to_streams_stream_id_stream_tag_id ON link_stream_tags_to_streams(stream_id, stream_tag_id);



/* Stored procedures for working with data from the "stream_tags" table. */

/* Update the "stream_tags" data for user. */
CREATE OR REPLACE PROCEDURE update_stream_tags_for_user(user_id1 INTEGER) 
LANGUAGE plpgsql
AS $$
DECLARE
  ids_remove INTEGER[];
BEGIN
   -- raise notice 'user_id1: %', user_id1;
  IF (user_id1 IS NULL) THEN
    RETURN;
  END IF;
  
  -- Get an array of identifiers of tag names that are no longer used.
  ids_remove := ARRAY(
    SELECT T.id
    FROM stream_tags T
    WHERE T.user_id = user_id1 AND
      NOT EXISTS(SELECT 1 FROM link_stream_tags_to_streams L WHERE  L.stream_tag_id = T.id LIMIT 1)
  );
   
  IF ARRAY_LENGTH(ids_remove, 1) > 0 THEN
    -- Removing tag names that are no longer used.
    DELETE
    FROM stream_tags
    WHERE id IN (SELECT UNNEST(ids_remove) AS id);
    -- raise notice 'DELETE FROM stream_tags() ids_remove2: %', ids_remove;
  END IF;

END;
$$;


/* Update links to the "tag" list for "stream". */
CREATE OR REPLACE PROCEDURE update_list_stream_tags(id1 INTEGER, user_id1 INTEGER, tags_new TEXT[]) 
LANGUAGE plpgsql
AS $$
DECLARE
  tags_old VARCHAR[];   tags_comm VARCHAR[];
  tags_add VARCHAR[];   tags_remove VARCHAR[];
  ids_remove INTEGER[]; tags_names_new VARCHAR[];
BEGIN
   -- raise notice 'id1: %, user_id1: %, tags_new: %', id1, user_id1, tags_new;
  IF (id1 IS NULL OR user_id1 IS NULL OR tags_new IS NULL) THEN
    RETURN;
  END IF;
  
  tags_old := ARRAY(
    SELECT T."name"
    FROM stream_tags T, link_stream_tags_to_streams L
    WHERE T.user_id = user_id1 AND T.id = L.stream_tag_id  AND L.stream_id = id1
  );
  -- raise notice 'tags_new: %, tags_old: %', tags_new, tags_old;
 
  -- Get common elements in both arrays
  tags_comm := ARRAY(SELECT UNNEST(tags_old) INTERSECT SELECT UNNEST(tags_new));
  -- Get the elements to be removed from an set.
  tags_remove := ARRAY(SELECT UNNEST(tags_old) EXCEPT SELECT UNNEST(tags_comm)); 
  -- Get the elements to be added to the set.
  tags_add := ARRAY(SELECT UNNEST(tags_new) EXCEPT SELECT UNNEST(tags_comm)); 
  -- raise notice 'tags_add: %, tags_remove: %', tags_add, tags_remove;

  -- Adding new elements
  IF ARRAY_LENGTH(tags_add, 1) > 0 THEN
    -- Get a list of tag names missing in the "stream_tags" table.
    tags_names_new := ARRAY(
      SELECT N."name"
      FROM (SELECT UNNEST(tags_add) AS "name") N
        LEFT JOIN stream_tags T ON T.user_id = user_id1 AND T."name" = N."name"
      WHERE T.id IS NULL
    );
    -- Add these missing tag names to the "stream_tags" table.
    IF ARRAY_LENGTH(tags_names_new, 1) > 0 THEN
      INSERT INTO stream_tags(user_id, "name")
      SELECT user_id1, N."name"
      FROM (SELECT UNNEST(tags_names_new) AS "name") N;
      -- raise notice 'INSERT INTO stream_tags() tags_names_new: %', tags_names_new;
    END IF;
   -- Add information on all new tag names to the links table "link_stream_tags_to_streams".
    INSERT INTO link_stream_tags_to_streams(stream_tag_id, stream_id)
    SELECT T.id, id1
    FROM stream_tags T, (SELECT UNNEST(tags_add) AS "name") N  
    WHERE T.user_id = user_id1 AND T."name" = N."name";
  END IF;
 
  -- Removing old elements
  IF ARRAY_LENGTH(tags_remove, 1) > 0 THEN
    -- Get an array of identifiers of legacy tag names.
    ids_remove := ARRAY(
      SELECT T.id FROM stream_tags T, (SELECT UNNEST(tags_remove) AS "name") N
      WHERE T.user_id = user_id1 AND T."name" = N."name"
    );
    -- raise notice 'ids_remove: %', ids_remove;
   
    -- Delete information about all obsolete tag names in the links table "link_stream_tags_to_streams".
    DELETE
    FROM link_stream_tags_to_streams L
    WHERE L.id IN (
      SELECT L.id FROM link_stream_tags_to_streams L, (SELECT UNNEST(ids_remove) AS id) I
      WHERE L.stream_id = id1 AND L.stream_tag_id = I.id
    );
    -- raise notice 'DELETE FROM link_stream_tags_to_streams() tags_remove: %', tags_remove;
    
    -- Update the "stream_tags" data for "stream".
    CALL update_stream_tags_for_user(user_id1);
  END IF;

END;
$$;


/* Stored function to retrieve data from the "stream_tags" table. */
CREATE OR REPLACE FUNCTION get_stream_tags_names(
  IN ids INTEGER[],
  OUT stream_id INTEGER, OUT id INTEGER, OUT user_id INTEGER, OUT "name" VARCHAR
) RETURNS SETOF record LANGUAGE sql
AS $$
  SELECT
    L.stream_id, T.id, T.user_id, T."name"
  FROM
    link_stream_tags_to_streams L,
    stream_tags T,
  (SELECT a AS id FROM unnest(ids) AS a) B
  WHERE
    L.stream_tag_id = T.id and L.stream_id = B.id
  ORDER BY
    L.stream_id ASC, T.user_id  ASC, T.id ASC;
$$;

/* Create a stored function to get information about the live of the stream. */
CREATE OR REPLACE FUNCTION get_stream_available(
  IN _stream_id INTEGER,
  OUT stream_id INTEGER,
  OUT stream_available BOOLEAN
) RETURNS SETOF record LANGUAGE plpgsql
AS $$
DECLARE
  rec1 RECORD;
BEGIN
  IF (_stream_id IS NULL) THEN
    RETURN;
  END IF;

  SELECT s.id AS stream_id, s.state != 'stopped' AS stream_available
  FROM streams s
  WHERE s.id = _stream_id
  INTO rec1;

  IF rec1.stream_available IS NULL THEN
    RETURN;
  END IF;

  RETURN QUERY SELECT
    rec1.stream_id,
    rec1.stream_available;
END;
$$;

-- **

/* Create a stored function that will filter "stream" entities by the specified parameters. */
CREATE OR REPLACE FUNCTION filter_streams(
  IN _id INTEGER,
  IN _user_id INTEGER,
  IN _is_logo BOOLEAN,
  IN _is_live BOOLEAN,
  OUT id INTEGER,
  OUT user_id INTEGER,
  OUT title VARCHAR,
  OUT descript TEXT,
  OUT logo VARCHAR,
  OUT starttime TIMESTAMP WITH TIME ZONE,
  OUT live BOOLEAN,
  OUT state stream_state,
  OUT started TIMESTAMP WITH TIME ZONE,
  OUT stopped TIMESTAMP WITH TIME ZONE,
  OUT source VARCHAR,
  OUT created_at TIMESTAMP WITH TIME ZONE,
  OUT updated_at TIMESTAMP WITH TIME ZONE
) RETURNS SETOF record LANGUAGE plpgsql
AS $$
BEGIN
  IF _id IS NULL AND _user_id IS NULL THEN
    RETURN;
  END IF;

  RETURN QUERY
    SELECT s.id, s.user_id, s.title, s.descript, s.logo, s.starttime, s.live, s.state,
      s.started, s.stopped, s.source, s.created_at, s.updated_at
    FROM streams s
    WHERE s.id = COALESCE(_id, s.id)
      AND s.user_id = COALESCE(_user_id, s.user_id)
      AND CASE WHEN _is_logo = true THEN LENGTH(COALESCE(s.logo, '')) > 0
          ELSE CASE WHEN _is_logo = false THEN LENGTH(COALESCE(s.logo, '')) = 0 ELSE true END
          END
      AND s.live = COALESCE(_is_live, s.live)
    ORDER BY s.id ASC;
END;
$$;

-- **
