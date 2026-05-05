-- ( 1 ) --

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
    /* The stream start time. Required on create */
    starttime TIMESTAMPTZ NOT NULL,
    /* Stream live status, false means inactive */
    live BOOLEAN NOT NULL DEFAULT FALSE,
    /* Stream live state - waiting (default), preparing, start, paused, stopped. */
    "state" stream_state DEFAULT 'waiting' NOT NULL,
    /* The time the stream began. */
    "started" TIMESTAMPTZ NULL,
    /* The time the stream began pausing. */
    "paused" TIMESTAMPTZ NULL,
    /* The time the stream stopped. */
    "stopped" TIMESTAMPTZ NULL,
    /* stream source */
    source VARCHAR(255) DEFAULT 'obs' NOT NULL,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL
);

SELECT diesel_manage_updated_at('streams');

CREATE INDEX idx_streams_user_id ON streams(user_id);
CREATE INDEX idx_streams_starttime ON streams(starttime);
CREATE INDEX idx_streams_live ON streams(live);
CREATE INDEX idx_streams_state ON streams("state");

/* Create trigger function for table "streams". */
CREATE OR REPLACE FUNCTION fn_bfr_upd_stream_set_live()
RETURNS TRIGGER 
LANGUAGE plpgsql 
AS $$
BEGIN
  NEW.live := NEW."state" IN ('preparing', 'started', 'paused');
  IF NEW."state" = 'started' AND NEW.started IS NULL THEN
    NEW.started := CURRENT_TIMESTAMP;
  END IF;
  IF NEW."state" = 'paused' THEN
    NEW.paused := CURRENT_TIMESTAMP;
  END IF;
  IF NEW."state" = 'stopped' THEN
    NEW.stopped := CURRENT_TIMESTAMP;
  END IF;
  RETURN NEW;
END;
$$;

/* Create trigger for table "streams". */
CREATE OR REPLACE TRIGGER trg_bfr_ins_stream_set_live
BEFORE INSERT ON streams
FOR EACH ROW
EXECUTE FUNCTION fn_bfr_upd_stream_set_live();

CREATE OR REPLACE TRIGGER trg_bfr_upd_stream_set_live
BEFORE UPDATE ON streams
FOR EACH ROW
EXECUTE FUNCTION fn_bfr_upd_stream_set_live();

-- ( 1 ) --
-- ( 2 ) --

/* Create "stream_tags" table. */
CREATE TABLE stream_tags (
    id SERIAL PRIMARY KEY NOT NULL,
    /* Custom tag name. */
    "name" VARCHAR(255) NOT NULL,
    /* Amount of links to stream tags. */
    count_links INTEGER NOT NULL DEFAULT 0
);

CREATE UNIQUE INDEX uq_idx_stream_tags_name ON stream_tags("name");
CREATE INDEX idx_stream_tags_count_links ON stream_tags(count_links);

-- ( 2 ) --
-- ( 3 ) --

/* Create "link_stream_tags_to_streams" table. */
CREATE TABLE link_stream_tags_to_streams (
    id SERIAL PRIMARY KEY NOT NULL,
    /* Stream Tag id */
    stream_tag_id INT NOT NULL REFERENCES stream_tags(id),
    /* Stream id */
    stream_id INT NOT NULL REFERENCES streams(id) ON DELETE CASCADE
);

CREATE UNIQUE INDEX uq_idx_link_stream_tags_to_streams ON link_stream_tags_to_streams(stream_id, stream_tag_id);


/* Create a function to increment a "count_links" to the "stream_tags" table
 after adding a record to the "link_stream_tags_to_streams" table. */
CREATE OR REPLACE FUNCTION fn_aft_ins_link_stream_tags_to_streams() RETURNS TRIGGER AS $$
BEGIN
  UPDATE stream_tags
  SET count_links = COALESCE(count_links, 0) + 1
  WHERE stream_tags.id = NEW.stream_tag_id;

  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

/* Create a trigger after adding a record to the "link_stream_tags_to_streams" table. */
CREATE TRIGGER trg_aft_ins_link_stream_tags_to_streams
  AFTER INSERT ON link_stream_tags_to_streams 
  FOR EACH ROW 
  EXECUTE PROCEDURE fn_aft_ins_link_stream_tags_to_streams();


/* Create a function to increment and decrement a "count_links" to the "stream_tags" table
 after update a record to the "link_stream_tags_to_streams" table. */
CREATE OR REPLACE FUNCTION fn_aft_upd_link_stream_tags_to_streams() RETURNS TRIGGER AS $$
BEGIN
  IF (OLD.stream_tag_id <> NEW.stream_tag_id) THEN

    UPDATE stream_tags
    SET count_links = COALESCE(count_links, 1) - 1
    WHERE stream_tags.id = OLD.stream_tag_id;

    UPDATE stream_tags
    SET count_links = COALESCE(count_links, 0) + 1
    WHERE stream_tags.id = NEW.stream_tag_id;
  END IF;

  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

/* Create a trigger after updating a record to the "link_stream_tags_to_streams" table. */
CREATE TRIGGER trg_aft_upd_link_stream_tags_to_streams
  AFTER UPDATE OF stream_tag_id ON link_stream_tags_to_streams 
  FOR EACH ROW 
  WHEN (OLD.stream_tag_id IS DISTINCT FROM NEW.stream_tag_id)
  EXECUTE PROCEDURE fn_aft_upd_link_stream_tags_to_streams();


/* Create a function to decrement a "count_links" to the "stream_tags" table
 after deleting a record to the "link_stream_tags_to_streams" table. */
CREATE OR REPLACE FUNCTION fn_aft_del_link_stream_tags_to_streams() RETURNS TRIGGER AS $$
BEGIN
  UPDATE stream_tags
  SET count_links = COALESCE(count_links, 1) - 1
  WHERE stream_tags.id = OLD.stream_tag_id;

  RETURN OLD;
END;
$$ LANGUAGE plpgsql;

/* Create a trigger after updating a record to the "link_stream_tags_to_streams" table. */
CREATE TRIGGER trg_aft_del_link_stream_tags_to_streams
  AFTER DELETE ON link_stream_tags_to_streams 
  FOR EACH ROW 
  EXECUTE PROCEDURE fn_aft_del_link_stream_tags_to_streams();

-- ( 3 ) --
-- ( 4 ) --

/* Create a stored function to modify the entry in "stream_tags". */
CREATE OR REPLACE FUNCTION modify_stream_tag(
  IN _stream_id INTEGER,
  IN _tags VARCHAR[],
  OUT tags VARCHAR[]
) LANGUAGE plpgsql
AS $$
DECLARE
  tags_old VARCHAR[];
  tags_comm VARCHAR[];
  tags_add VARCHAR[] := ARRAY[]::VARCHAR[];
  tags_remove VARCHAR[] := ARRAY[]::VARCHAR[];
  tags_add_new VARCHAR[];
BEGIN
  -- RAISE NOTICE 'm.s.t() _stream_id: %, _tags: %', _stream_id, _tags;
  IF (_stream_id IS NULL) THEN
    -- RAISE NOTICE 'm.s.t() _stream_id IS NULL Exit;';
    RETURN;
  END IF;

  -- Check for the presence of a record in the main table.  
  IF (NOT EXISTS (SELECT s.id FROM streams s WHERE s.id = _stream_id)) THEN
    -- RAISE NOTICE 'm.s.t() IS NOT EXISTS _stream_id Exit;';
    RETURN;
  END IF;
  
  tags_old := ARRAY(
    SELECT st."name"
    FROM link_stream_tags_to_streams l, stream_tags st
    WHERE l.stream_id = _stream_id AND l.stream_tag_id = st.id
    ORDER BY st.id
  );
  -- RAISE NOTICE 'm.s.t() tags_old: %', tags_old;

  IF (_tags IS NULL) THEN 
    tags := tags_old;
    -- RAISE NOTICE 'm.s.t() tags := tags_old; Exit;';
    RETURN;
  END IF;

  -- Get common elements in both arrays
  tags_comm := ARRAY(SELECT UNNEST(tags_old) INTERSECT SELECT UNNEST(_tags));
  -- RAISE NOTICE 'm.s.t() tags_comm: %', tags_comm;
  -- Get the elements to be removed from an set.
  tags_remove := ARRAY(SELECT UNNEST(tags_old) EXCEPT SELECT UNNEST(tags_comm)); 
  -- RAISE NOTICE 'm.s.t() tags_remove: %', tags_remove;
  -- Get the elements to be added to the set.
  tags_add := ARRAY(SELECT UNNEST(_tags) EXCEPT SELECT UNNEST(tags_comm)); 
  -- RAISE NOTICE 'm.s.t() tags_add: %', tags_add;

  -- Adding new elements
  IF (ARRAY_LENGTH(tags_add, 1) > 0) THEN
    -- Get a list of tag names missing in the "stream_tags" table.
    tags_add_new := ARRAY(
      SELECT n."name"
      FROM (SELECT UNNEST(tags_add) AS "name") n
        LEFT JOIN stream_tags st ON st."name" = n."name"
      WHERE st.id IS NULL
      ORDER BY "name"
    );
    -- RAISE NOTICE 'm.s.t() tags_add_new: %', tags_add_new;

    -- Add these missing tag names to the "stream_tags" table.
    IF (ARRAY_LENGTH(tags_add_new, 1) > 0) THEN
      INSERT INTO stream_tags("name")
      SELECT n."name"
      FROM (SELECT UNNEST(tags_add_new) AS "name") n;
      -- RAISE NOTICE 'm.s.t() INSERT INTO stream_tags() tags_add_new: %', tags_add_new;
    END IF;
   -- Add information on all new tag names to the links table "link_stream_tags_to_streams".
    INSERT INTO link_stream_tags_to_streams(stream_tag_id, stream_id)
    SELECT st.id, _stream_id
    FROM stream_tags st, (SELECT UNNEST(tags_add) AS "name") n  
    WHERE st."name" = n."name";
  END IF;

  -- Removing old elements
  IF (ARRAY_LENGTH(tags_remove, 1) > 0) THEN
    -- Delete information about all obsolete tag names in the links table "link_stream_tags_to_streams".
    DELETE
    FROM link_stream_tags_to_streams
    WHERE id IN (
      SELECT l.id
      FROM link_stream_tags_to_streams l,
           stream_tags st,
          (SELECT UNNEST(tags_remove) AS "name") n
      WHERE l.stream_id = _stream_id
        AND l.stream_tag_id = st.id
        AND st."name" = n."name"
    );

    -- Delete entries for all tag in stream_tags that no longer have "link_stream_tags_to_streams" links.
    PERFORM delete2_stream_tag_deprecated(tags_remove);
  END IF;

  tags := ARRAY(
    SELECT st."name"
    FROM link_stream_tags_to_streams l, stream_tags st
    WHERE l.stream_id = _stream_id AND l.stream_tag_id = st.id
    ORDER BY st.id
  );
  -- RAISE NOTICE 'm.s.t() tags: %', tags;

  RETURN;
END;
$$;


/* Create a stored function to delete the entity in "streams" and "stream_tags". */
CREATE OR REPLACE FUNCTION delete_stream_tag_deprecated(
  IN _tags VARCHAR[],
  OUT "count" INTEGER
) LANGUAGE plpgsql
AS $$
BEGIN
  IF (_tags IS NULL OR ARRAY_LENGTH(_tags, 1) = 0) THEN
    -- RAISE NOTICE 'd.s.t.d() _tags IS NULL OR _tags=[]';
    RETURN;
  END IF;

  -- Delete entries for all tag in stream_tags that no longer have "link_stream_tags_to_streams" links.
  DELETE
  FROM stream_tags st
  WHERE st."name" IN (SELECT UNNEST(_tags) AS "name")
    AND NOT EXISTS(SELECT 1 FROM link_stream_tags_to_streams l WHERE l.stream_tag_id = st.id LIMIT 1);
  -- Get the number of deleted records.
  GET DIAGNOSTICS "count" = ROW_COUNT;
  -- RAISE NOTICE 'd.s.t.d(_tags: %) count: %', _tags, "count";

  RETURN;
END;
$$;

/* Create a stored function to add a new entry to "streams" and "stream_tags". */
CREATE OR REPLACE FUNCTION create_stream_and_stream_tag(
  IN _user_id INTEGER, -- (required)
  IN _title VARCHAR, -- (required)
  IN _descript TEXT,
  IN _logo VARCHAR,
  IN _starttime TIMESTAMPTZ, -- (required)
  IN _state stream_state,
  IN _started TIMESTAMPTZ,
  IN _paused TIMESTAMPTZ,
  IN _stopped TIMESTAMPTZ,
  IN _source VARCHAR,
  IN _tags VARCHAR[], -- (required)
  OUT id INTEGER,
  OUT user_id INTEGER,
  OUT title VARCHAR,
  OUT descript TEXT,
  OUT logo VARCHAR,
  OUT starttime TIMESTAMPTZ,
  OUT live BOOLEAN,
  OUT "state" stream_state,
  OUT "started" TIMESTAMPTZ,
  OUT "paused" TIMESTAMPTZ,
  OUT "stopped" TIMESTAMPTZ,
  OUT source VARCHAR,
  OUT created_at TIMESTAMPTZ,
  OUT updated_at TIMESTAMPTZ,
  OUT tags VARCHAR[],
  OUT old_logo VARCHAR
) RETURNS SETOF record LANGUAGE plpgsql
AS $$
DECLARE 
  rec1 RECORD;
BEGIN
  -- RAISE NOTICE 'c.s.a.s.t() _user_id: %, _title: %, _starttime: %, tags: %', _user_id, _title, _starttime, _tags;
  IF (_user_id IS NULL OR _title IS NULL OR _starttime IS NULL OR _tags IS NULL OR ARRAY_LENGTH(_tags, 1) = 0) THEN
    RETURN;
  END IF;

  -- Add a new entry to the "streams" table.
  INSERT INTO streams (
    user_id, title, descript, logo, starttime, state,
    started, paused, stopped, "source")
  VALUES(
    _user_id, _title, COALESCE(_descript, ''), _logo, _starttime, COALESCE(_state, 'waiting'::stream_state), 
    _started, _paused, _stopped, COALESCE(_source, 'obs'::character varying))
  RETURNING
    streams.id, streams.user_id, streams.title, streams.descript, streams.logo, streams.starttime,
    streams.live, streams."state", streams."started", streams."paused", streams."stopped",
    streams.source, streams.created_at, streams.updated_at
  INTO rec1;
  -- RAISE NOTICE 'c.s.a.s.t() rec1.id: %', rec1.id;

  IF rec1 IS NULL OR rec1.id IS NULL THEN
    -- RAISE NOTICE 'c.s.a.s.t() rec1.id IS NULL';
    RETURN;
  END IF;

  tags := modify_stream_tag(rec1.id, _tags);

  RETURN QUERY SELECT
    rec1.id, rec1.user_id, rec1.title, rec1.descript, rec1.logo, rec1.starttime,
    rec1.live, rec1."state", rec1."started", rec1."paused", rec1."stopped",
    rec1.source, rec1.created_at, rec1.updated_at, tags, old_logo;
END;
$$;


/* Create a stored function to modify the entry in "streams" and "stream_tags". */
CREATE OR REPLACE FUNCTION modify_stream_and_stream_tag(
  IN _id INTEGER, -- (required)
  IN _user_id INTEGER, -- (required)
  IN _title VARCHAR,
  IN _descript TEXT,
  IN _logo VARCHAR,
  IN _starttime TIMESTAMPTZ,
  IN _state stream_state,
  IN _started TIMESTAMPTZ,
  IN _paused TIMESTAMPTZ,
  IN _stopped TIMESTAMPTZ,
  IN _source VARCHAR,
  IN _tags VARCHAR[],
  OUT id INTEGER,
  OUT user_id INTEGER,
  OUT title VARCHAR,
  OUT descript TEXT,
  OUT logo VARCHAR,
  OUT starttime TIMESTAMPTZ,
  OUT live BOOLEAN,
  OUT "state" stream_state,
  OUT "started" TIMESTAMPTZ,
  OUT "paused" TIMESTAMPTZ,
  OUT "stopped" TIMESTAMPTZ,
  OUT source VARCHAR,
  OUT created_at TIMESTAMPTZ,
  OUT updated_at TIMESTAMPTZ,
  OUT tags VARCHAR[],
  OUT old_logo VARCHAR
) RETURNS SETOF record LANGUAGE plpgsql
AS $$
DECLARE
  rec1 RECORD;
  is_changed BOOLEAN;
BEGIN
  -- RAISE NOTICE 'm.s.a.s.t() _id: %, _user_id: %', _id, _user_id;
  IF (_id IS NULL OR _user_id IS NULL) THEN
    RETURN;
  END IF;

  SELECT s.logo INTO old_logo FROM streams s WHERE s.id = _id;

  UPDATE streams set
    title     = CASE WHEN _title     IS NOT NULL THEN _title     ELSE streams.title     END,
    descript  = CASE WHEN _descript  IS NOT NULL THEN _descript  ELSE streams.descript  END,
    logo      = CASE WHEN _logo      IS NOT NULL THEN _logo      ELSE streams.logo      END,
    starttime = CASE WHEN _starttime IS NOT NULL THEN _starttime ELSE streams.starttime END,
    state     = CASE WHEN _state     IS NOT NULL THEN _state     ELSE streams.state     END,
    started   = CASE WHEN _started   IS NOT NULL THEN _started   ELSE streams.started   END,
    paused    = CASE WHEN _paused    IS NOT NULL THEN _paused    ELSE streams.paused    END,
    stopped   = CASE WHEN _stopped   IS NOT NULL THEN _stopped   ELSE streams.stopped   END,
    source    = CASE WHEN _source    IS NOT NULL THEN _source    ELSE streams.source    END
  WHERE streams.id = _id
    AND streams.user_id = _user_id
  RETURNING
    streams.id, streams.user_id, streams.title, streams.descript, streams.logo, streams.starttime,
    streams.live, streams."state", streams."started", streams."paused", streams."stopped",
    streams.source, streams.created_at, streams.updated_at
  INTO rec1;
  -- RAISE NOTICE 'm.s.a.s.t() rec1.id: %', rec1.id;
  
  IF (rec1 IS NULL OR rec1.id IS NULL) THEN
    -- RAISE NOTICE 'm.s.a.s.t() rec1.id IS NULL';
    RETURN;
  END IF;

  tags := modify_stream_tag(rec1.id, _tags);

  RETURN QUERY SELECT
    rec1.id, rec1.user_id, rec1.title, rec1.descript, rec1.logo, rec1.starttime,
    rec1.live, rec1."state", rec1."started", rec1."paused", rec1."stopped",
    rec1.source, rec1.created_at, rec1.updated_at, tags, old_logo;
END;
$$;


/* Create a stored function to delete the entity in "streams" and "stream_tags". */
CREATE OR REPLACE FUNCTION delete_stream_and_stream_tag(
  IN _id INTEGER,
  IN _user_id INTEGER,
  OUT id INTEGER,
  OUT user_id INTEGER,
  OUT title VARCHAR,
  OUT descript TEXT,
  OUT logo VARCHAR,
  OUT starttime TIMESTAMPTZ,
  OUT live BOOLEAN,
  OUT "state" stream_state,
  OUT "started" TIMESTAMPTZ,
  OUT "paused" TIMESTAMPTZ,
  OUT "stopped" TIMESTAMPTZ,
  OUT source VARCHAR,
  OUT created_at TIMESTAMPTZ,
  OUT updated_at TIMESTAMPTZ,
  OUT tags VARCHAR[],
  OUT old_logo VARCHAR
) RETURNS SETOF record LANGUAGE plpgsql
AS $$
DECLARE
  rec1 RECORD;
BEGIN
  -- RAISE NOTICE 'd.s.a.s.t() _id: %, _user_id: %', _id, _user_id;
  IF (_id IS NULL OR _user_id IS NULL) THEN
    RETURN;
  END IF;

  -- Get a list of tags for this stream.
  tags := ARRAY(
    SELECT st."name"
    FROM link_stream_tags_to_streams l, stream_tags st
    WHERE l.stream_id = _id AND l.stream_tag_id = st.id
    ORDER BY st.id
  );

  DELETE FROM streams
  WHERE streams.id = _id
    AND streams.user_id = _user_id
  RETURNING
    streams.id, streams.user_id, streams.title, streams.descript, streams.logo, streams.starttime,
    streams.live, streams."state", streams."started", streams."paused", streams."stopped",
    streams.source, streams.created_at, streams.updated_at
  INTO rec1;
  -- RAISE NOTICE 'd.s.a.s.t() rec1.id: %', rec1.id;

  IF (rec1.id IS NULL) THEN
    -- RAISE NOTICE 'd.s.a.s.t() rec1.id IS NULL';
    tags := NULL;
    RETURN;
  END IF;

  -- Delete entries for all tag in stream_tags that no longer have "link_stream_tags_to_streams" links.
  PERFORM delete_stream_tag_deprecated(tags);

  RETURN QUERY SELECT
    rec1.id, rec1.user_id, rec1.title, rec1.descript, rec1.logo, rec1.starttime,
    rec1.live, rec1."state", rec1."started", rec1."paused", rec1."stopped",
    rec1.source, rec1.created_at, rec1.updated_at, tags, old_logo;
END;
$$;

-- ( 4 ) --
-- ( 5 ) --

/* Create a stored function to get an entity from the "streams" and "stream_tags". */
CREATE OR REPLACE FUNCTION get_stream_and_stream_tag(
  IN _id INTEGER,
  OUT id INTEGER,
  OUT user_id INTEGER,
  OUT title VARCHAR,
  OUT descript TEXT,
  OUT logo VARCHAR,
  OUT starttime TIMESTAMPTZ,
  OUT live BOOLEAN,
  OUT "state" stream_state,
  OUT "started" TIMESTAMPTZ,
  OUT "paused" TIMESTAMPTZ,
  OUT "stopped" TIMESTAMPTZ,
  OUT source VARCHAR,
  OUT created_at TIMESTAMPTZ,
  OUT updated_at TIMESTAMPTZ,
  OUT old_logo VARCHAR,
  OUT tags VARCHAR[]
) RETURNS SETOF record LANGUAGE plpgsql
AS $$
DECLARE
  rec1 RECORD;
BEGIN
  -- RAISE NOTICE 'g.s.a.s.t() _id: %', _id;
  IF (_id IS NULL) THEN
    RETURN;
  END IF;

  -- Get a list of tags for this stream.
  tags := ARRAY(
    SELECT st."name"
    FROM link_stream_tags_to_streams l, stream_tags st
    WHERE l.stream_id = _id AND l.stream_tag_id = st.id
    ORDER BY st.id
  );

  SELECT
    s.id, s.user_id, s.title, s.descript, s.logo, s.starttime,
    s.live, s."state", s."started", s."paused", s."stopped",
    s.source, s.created_at, s.updated_at 
  FROM streams s
  WHERE s.id = _id
  INTO rec1;
  -- RAISE NOTICE 'g.s.a.s.t() rec1.id: %', rec1.id;

  IF (rec1.id IS NULL) THEN
    -- RAISE NOTICE 'g.s.a.s.t() rec1.id IS NULL';
    tags := ARRAY[]::VARCHAR[];
    RETURN;
  END IF;

  RETURN QUERY SELECT
    rec1.id, rec1.user_id, rec1.title, rec1.descript, rec1.logo, rec1.starttime,
    rec1.live, rec1."state", rec1."started", rec1."paused", rec1."stopped",
    rec1.source, rec1.created_at, rec1.updated_at, tags, old_logo;
END;
$$;

/* Create a stored function to filter the entity from "streams" and "stream_tags" according to the conditions. */
CREATE OR REPLACE FUNCTION filter_stream_and_stream_tag_by_pages_ids(
  IN _user_id INTEGER,
  IN _live BOOLEAN,
  IN _filter VARCHAR, -- 'future', 'past', 'period'
  IN _starttime TIMESTAMPTZ,
  IN _finishtime TIMESTAMPTZ,
  IN _sort_desc BOOLEAN,
  IN _tag VARCHAR,
  IN _limit INTEGER,
  IN _offset INTEGER,
  OUT id INTEGER
) RETURNS SETOF INTEGER LANGUAGE plpgsql
AS $$
BEGIN
  -- RAISE NOTICE 'f.s.a.s.t.b.p.i() _user_id: %, _live: %, _filter: %', _user_id, _live, _filter;
  -- RAISE NOTICE 'f.s.a.s.t.b.p.i() _starttime: %, _finishtime: %, _sort_desc: %', _starttime, _finishtime, _sort_desc;
  -- RAISE NOTICE 'f.s.a.s.t.b.p.i() _tag: %, _limit: %, _offset: %', _tag, _limit, _offset;
  IF (_filter IN ('future', 'past') AND _starttime IS NULL) THEN
    _starttime := CURRENT_TIMESTAMP;
  END IF;
  IF (_filter = 'period' AND _starttime IS NULL AND _finishtime IS NULL) THEN
    RETURN;
  END IF;

  IF _tag IS NULL OR LENGTH(_tag) = 0 THEN
    RETURN QUERY
      SELECT s.id
      FROM streams s
      WHERE s.user_id = COALESCE(_user_id, s.user_id)
        AND s.live = COALESCE(_live, s.live)
        AND CASE
              WHEN _filter = 'future' THEN s.starttime >= _starttime
              WHEN _filter = 'past'   THEN s.starttime < _starttime
              WHEN _filter = 'period' THEN _starttime <= s.starttime AND s.starttime <= _finishtime
              ELSE true
            END
      ORDER BY
        CASE WHEN _sort_desc = true                   THEN s.starttime ELSE NULL END DESC,
        CASE WHEN COALESCE(_sort_desc, false) = false THEN s.starttime ELSE NULL END ASC,
        s.id ASC
      LIMIT _limit
      OFFSET _offset;
  ELSE
    RETURN QUERY
      SELECT s.id
      FROM streams s
        LEFT JOIN (link_stream_tags_to_streams l JOIN stream_tags st ON (l.stream_tag_id = st.id)) ON (s.id = l.stream_id)
      WHERE st.name = _tag 
        AND s.user_id = COALESCE(_user_id, s.user_id)
        AND s.live = COALESCE(_live, s.live)
        AND CASE
              WHEN _filter = 'future' THEN s.starttime >= _starttime
              WHEN _filter = 'past'   THEN s.starttime < _starttime
              WHEN _filter = 'period'  THEN _starttime <= s.starttime AND s.starttime <= _finishtime
              ELSE true
            END
      ORDER BY
        CASE WHEN _sort_desc = true                   THEN s.starttime ELSE NULL END DESC,
        CASE WHEN COALESCE(_sort_desc, false) = false THEN s.starttime ELSE NULL END ASC,
        s.id ASC
      LIMIT _limit
      OFFSET _offset;
  END IF;
END;
$$;

/* Create a stored function to filter the entity from "streams" and "stream_tags" according to the conditions. */
CREATE OR REPLACE FUNCTION filter_stream_and_stream_tag_by_pages_list(
  IN _user_id INTEGER,
  IN _live BOOLEAN,
  IN _filter VARCHAR, -- 'future', 'past', 'period'
  IN _starttime TIMESTAMPTZ,
  IN _finishtime TIMESTAMPTZ,
  IN _sort_desc BOOLEAN,
  IN _tag VARCHAR,
  IN _limit INTEGER,
  IN _offset INTEGER,
  OUT id INTEGER,
  OUT user_id INTEGER,
  OUT title VARCHAR,
  OUT descript TEXT,
  OUT logo VARCHAR,
  OUT starttime TIMESTAMPTZ,
  OUT live BOOLEAN,
  OUT "state" stream_state,
  OUT "started" TIMESTAMPTZ,
  OUT "paused" TIMESTAMPTZ,
  OUT "stopped" TIMESTAMPTZ,
  OUT source VARCHAR,
  OUT created_at TIMESTAMPTZ,
  OUT updated_at TIMESTAMPTZ,
  OUT old_logo VARCHAR,
  OUT tags VARCHAR[]
) RETURNS SETOF record LANGUAGE plpgsql
AS $$
DECLARE
  rec1 RECORD;
  mark_ids INTEGER[] := ARRAY[]::INTEGER[];
  mark_id INTEGER;
  idx1 INTEGER;
  len1 INTEGER;
BEGIN
  -- RAISE NOTICE 'f.s.a.s.t.b.p.l() _user_id: %, _live: %, _filter: %', _user_id, _live, _filter;
  -- RAISE NOTICE 'f.s.a.s.t.b.p.l() _starttime: %, _finishtime: %, _sort_desc: %', _starttime, _finishtime, _sort_desc;
  -- RAISE NOTICE 'f.s.a.s.t.b.p.l() _tag: %, _limit: %, _offset: %', _tag, _limit, _offset;
  IF (_filter IS NOT NULL AND _starttime IS NULL) THEN
    _starttime := CURRENT_TIMESTAMP;
  END IF;

  FOR rec1 IN
    SELECT f.id 
    FROM filter_stream_and_stream_tag_by_pages_ids(_user_id, _live, _filter, _starttime, _finishtime, _sort_desc, _tag, _limit, _offset) f
  LOOP
    mark_ids := mark_ids || rec1.id;
  END LOOP;

  len1 := ARRAY_LENGTH(mark_ids, 1);
  idx1 := 1;
  WHILE idx1 <= len1 LOOP
    mark_id = mark_ids[idx1];

    tags := ARRAY(
      SELECT st."name"
      FROM link_stream_tags_to_streams l, stream_tags st
      WHERE l.stream_id = mark_id AND l.stream_tag_id = st.id
      ORDER BY st.id
    );
    -- RAISE NOTICE 'f.s.a.s.t.b.p.l() mark_id: %, tags: %', mark_id, tags;

    RETURN QUERY 
    SELECT s.id, s.user_id, s.title, s.descript, s.logo, s.starttime, s.live, s."state",
      s."started", s."paused", s."stopped", s.source, s.created_at, s.updated_at, old_logo, tags
    FROM streams s
    WHERE s.id = mark_id;

    idx1 := idx1 + 1;
  END LOOP;
END;
$$;

/* Create a stored function to filter the entity from "streams" and "stream_tags" according to the conditions. */
CREATE OR REPLACE FUNCTION filter_stream_and_stream_tag_by_pages_count(
  IN _user_id INTEGER,
  IN _live BOOLEAN,
  IN _filter VARCHAR, -- 'future', 'past', 'period'
  IN _starttime TIMESTAMPTZ,
  IN _finishtime TIMESTAMPTZ,
  IN _tag VARCHAR
) RETURNS INTEGER LANGUAGE plpgsql
AS $$
DECLARE
   _cnt INTEGER := -1;
BEGIN
  -- RAISE NOTICE 'f.s.a.s.t.b.p.c() _user_id: %, _live: %, _filter: %', _user_id, _live, _filter;
  -- RAISE NOTICE 'f.s.a.s.t.b.p.c() _starttime: %, _finishtime: %, _tag: %', _starttime, _finishtime, _tag;
  IF (_filter IN ('future', 'past') AND _starttime IS NULL) THEN
    _starttime := CURRENT_TIMESTAMP;
  END IF;
  IF (_filter = 'period' AND _starttime IS NULL AND _finishtime IS NULL) THEN
    RETURN NULL;
  END IF;

  IF _tag IS NULL OR LENGTH(_tag) = 0 THEN
    SELECT count(s.id)
    INTO _cnt
    FROM streams s
    WHERE s.user_id = COALESCE(_user_id, s.user_id)
      AND s.live = COALESCE(_live, s.live)
      AND CASE
            WHEN _filter = 'future' THEN s.starttime >= _starttime
            WHEN _filter = 'past'   THEN s.starttime < _starttime
            WHEN _filter = 'period'  THEN _starttime <= s.starttime AND s.starttime <= _finishtime
            ELSE true
          END;
  ELSE
    SELECT count(s.id)
    INTO _cnt
    FROM streams s
      LEFT JOIN (link_stream_tags_to_streams l JOIN stream_tags st ON (l.stream_tag_id = st.id)) ON (s.id = l.stream_id)
    WHERE st.name = _tag
      AND s.user_id = COALESCE(_user_id, s.user_id)
      AND s.live = COALESCE(_live, s.live)
      AND CASE
            WHEN _filter = 'future' THEN s.starttime >= _starttime
            WHEN _filter = 'past'   THEN s.starttime < _starttime
            WHEN _filter = 'period'  THEN _starttime <= s.starttime AND s.starttime <= _finishtime
            ELSE true
          END;
  END IF;
   -- RAISE NOTICE 'f.s.a.s.t.b.p.c() _cnt: %', _cnt;

  RETURN _cnt;
END;
$$;

/* Create a stored function to get an entity from the "stream_tags". */
CREATE OR REPLACE FUNCTION get_stream_tags(
  IN _sort_column VARCHAR, -- 'id','name','count_links'
  IN _sort_desc BOOLEAN,
  IN _limit INTEGER,
  IN _offset INTEGER,
  OUT id INTEGER,
  OUT "name" VARCHAR,
  OUT count_links INTEGER
) RETURNS SETOF record LANGUAGE plpgsql
AS $$
BEGIN
  IF (_sort_column IS NULL) THEN
    _sort_column := 'id';
  END IF;
  IF (_sort_desc IS NULL) THEN
    _sort_desc := false;
  END IF;

  RETURN QUERY
    SELECT
      st.id, st."name", st.count_links
    FROM
      stream_tags st 
    ORDER BY
      CASE WHEN _sort_column = 'id' AND COALESCE(_sort_desc, false) = false THEN st.id ELSE NULL END ASC,
      CASE WHEN _sort_column = 'id' AND _sort_desc = true  THEN st.id ELSE NULL END DESC,
      CASE WHEN _sort_column = 'name' AND COALESCE(_sort_desc, false) = false THEN st."name" ELSE NULL END ASC,
      CASE WHEN _sort_column = 'name' AND _sort_desc = true  THEN st."name" ELSE NULL END DESC,
      CASE WHEN _sort_column = 'countlinks' AND COALESCE(_sort_desc, false) = false THEN st.count_links ELSE NULL END ASC,
      CASE WHEN _sort_column = 'countlinks' AND _sort_desc = true  THEN st.count_links ELSE NULL END DESC
    LIMIT _limit
    OFFSET _offset;
END;
$$;

-- ( 5 ) --
