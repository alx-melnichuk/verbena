
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
CREATE INDEX idx_streams_starttime ON streams(starttime);
CREATE INDEX idx_streams_live ON streams(live);
CREATE INDEX idx_streams_status ON streams("status");


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

/* Update links to the "tag" list for "stream". */
CREATE OR REPLACE PROCEDURE update_list_stream_tag_to_stream(
  id1 INTEGER, user_id1 INTEGER, tag_name_list1 VARCHAR
) LANGUAGE plpgsql
AS $$
DECLARE
  tags_old VARCHAR[];   tags_new VARCHAR[];    tags_comm VARCHAR[];
  tags_add VARCHAR[];   tags_remove VARCHAR[];
  ids_remove INTEGER[]; tags_names_new VARCHAR[];
BEGIN
  -- raise notice '';
  IF (id1 IS NULL OR user_id1 IS NULL OR tag_name_list1 IS NULL) THEN
    RETURN;
  END IF;

  tags_new := STRING_TO_ARRAY(tag_name_list1, '@@~~##');
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
   
  END IF;

END;
$$;
