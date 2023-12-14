
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


/* Create "view_stream_tags_by_streams" view. */

CREATE OR REPLACE VIEW view_stream_tags_by_streams
AS
SELECT
  L.stream_id, T.user_id, T."name", L.stream_tag_id
FROM 
  link_stream_tags_to_streams L,
  stream_tags T
WHERE L.stream_tag_id = T.id;


/*
SELECT
  L.stream_id,
FROM 
  link_stream_tags_to_streams L
    INNER JOIN stream_tags T USING (stream_tag_id)
*/
