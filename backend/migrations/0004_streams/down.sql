/* Drop stored procedures for working with data from the "stream_tags" table. */
DROP PROCEDURE IF EXISTS update_list_stream_tag_to_stream;

/* Remove the indexes on the "link_stream_tags_to_streams" table. */
DROP INDEX IF EXISTS idx_link_stream_tags_to_streams_stream_id_stream_tag_id;

/* Remove the "link_stream_tags_to_streams" table. */
DROP TABLE IF EXISTS link_stream_tags_to_streams;


/* Remove the indexes on the "stream_tags" table. */
DROP INDEX IF EXISTS uq_idx_stream_tags_user_id_name;

/* Remove the "stream_tags" table. */
DROP TABLE IF EXISTS stream_tags;


/* Remove the indexes on the "streams" table. */
DROP INDEX IF EXISTS idx_streams_user_id;
DROP INDEX IF EXISTS idx_streams_live;
DROP INDEX IF EXISTS idx_streams_status;

/* Remove the "streams" table. */
DROP TABLE IF EXISTS streams;

/* Remove the "stream_state" type. */
DROP TYPE IF EXISTS stream_state;
