/* Remove the indexes on the "streams" table. */
DROP INDEX IF EXISTS idx_streams_user_id;
DROP INDEX IF EXISTS idx_streams_live;
DROP INDEX IF EXISTS idx_streams_status;

/* Remove the "streams" table. */
DROP TABLE IF EXISTS streams;

/* Remove the "stream_state" type. */
DROP TYPE IF EXISTS stream_state;
