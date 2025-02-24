/* Drop stored procedures for working with data from the "stream_tags" table. */
DROP PROCEDURE IF EXISTS update_list_stream_tags;
DROP PROCEDURE IF EXISTS update_stream_tags_for_user;

/* Drop stored function to retrieve data from the "stream_tags" table. */
DROP FUNCTION IF EXISTS get_stream_tags_names;

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
DROP INDEX IF EXISTS idx_streams_starttime;
DROP INDEX IF EXISTS idx_streams_live;
DROP INDEX IF EXISTS idx_streams_state;

/* Remove trigger for table "streams". */
DROP TRIGGER IF EXISTS tr_before_insert_stream_set_live ON streams;
DROP TRIGGER IF EXISTS tr_before_update_stream_set_live ON streams;
/* Remove trigger function for table "streams". */
DROP FUNCTION IF EXISTS modify_stream_set_live;

/* Remove the "streams" table. */
DROP TABLE IF EXISTS streams;

/* Remove the "stream_state" type. */
DROP TYPE IF EXISTS stream_state;
