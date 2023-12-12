
/* Create a type "stream_state". */
CREATE TYPE stream_state AS ENUM ('waiting', 'started', 'stopped', 'paused', 'preparing');

/* Create "streams" tables. */
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
