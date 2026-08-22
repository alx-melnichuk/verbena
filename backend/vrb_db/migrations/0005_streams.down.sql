-- Remove entities: "streams", "stream_tags", "link_stream_tags_to_streams".

-- ( 5 ) --

/* Remove stored function to get an entity from the "stream_tags". */
DROP FUNCTION IF EXISTS get_stream_tags;

/* Remove stored function to filter the entity from "streams" and "stream_tags" according to the conditions. */
DROP FUNCTION IF EXISTS filter_stream_and_stream_tag_by_pages_count;

/* Remove stored function to filter the entity from "streams" and "stream_tags" according to the conditions. */
DROP FUNCTION IF EXISTS filter_stream_and_stream_tag_by_pages_list;

/* Remove stored function to filter the entity from "streams" and "stream_tags" according to the conditions. */
DROP FUNCTION IF EXISTS filter_stream_and_stream_tag_by_pages_ids;

/* Remove stored function to get an entity from the "streams" and "stream_tags". */
DROP FUNCTION IF EXISTS get_stream_and_stream_tag;

-- ( 5 ) --
-- ( 4 ) --

/* Remove stored function to delete the entity in "streams" and "stream_tags". */
DROP FUNCTION IF EXISTS delete_stream_and_stream_tag;

/* Remove stored function to modify the entry in "streams" and "stream_tags". */
DROP FUNCTION IF EXISTS modify_stream_and_stream_tag;

/* Remove stored function to add a new entry to "streams" and "stream_tags". */
DROP FUNCTION IF EXISTS create_stream_and_stream_tag;

/* Remove stored function to delete the entity in "streams" and "stream_tags". */
DROP FUNCTION IF EXISTS delete_stream_tag_deprecated;

/* Remove stored function to modify the entry in "stream_tags". */
DROP FUNCTION IF EXISTS modify_stream_tag;

-- ( 4 ) --
-- ( 3 ) --

/* Remove trigger "trg_aft_del_link_stream_tags_to_streams". */
DROP TRIGGER IF EXISTS trg_aft_del_link_stream_tags_to_streams ON link_stream_tags_to_streams;
/* Remove function "fn_aft_del_link_stream_tags_to_streams". */
DROP FUNCTION IF EXISTS fn_aft_del_link_stream_tags_to_streams;


/* Remove trigger "trg_aft_upd_link_stream_tags_to_streams". */
DROP TRIGGER IF EXISTS trg_aft_upd_link_stream_tags_to_streams ON link_stream_tags_to_streams;
/* Remove function "fn_aft_upd_link_stream_tags_to_streams". */
DROP FUNCTION IF EXISTS fn_aft_upd_link_stream_tags_to_streams;


/* Remove trigger "trg_aft_ins_link_stream_tags_to_streams". */
DROP TRIGGER IF EXISTS trg_aft_ins_link_stream_tags_to_streams ON link_stream_tags_to_streams;
/* Remove function "fn_aft_ins_link_stream_tags_to_streams". */
DROP FUNCTION IF EXISTS fn_aft_ins_link_stream_tags_to_streams;

/* Remove indexes for the "link_stream_tags_to_streams" table. */
DROP INDEX IF EXISTS uq_idx_link_stream_tags_to_streams;

/* Remove the "link_stream_tags_to_streams" table. */
DROP TABLE IF EXISTS link_stream_tags_to_streams;

-- ( 3 ) --
-- ( 2 ) --

/* Remove indexes for the "stream_tags" table. */
DROP INDEX IF EXISTS idx_stream_tags_count_links;
DROP INDEX IF EXISTS uq_idx_stream_tags_name;

/* Remove the "stream_tags" table. */
DROP TABLE IF EXISTS stream_tags;

-- ( 2 ) --
-- ( 1 ) --

/* Remove trigger for table "streams". */
DROP TRIGGER IF EXISTS trg_bfr_upd_stream_set_live ON streams;
DROP TRIGGER IF EXISTS trg_bfr_ins_stream_set_live ON streams;
/* Remove trigger function for table "streams". */
DROP FUNCTION IF EXISTS fn_bfr_upd_stream_set_live;

/* Remove indexes for the "streams" table. */
DROP INDEX IF EXISTS idx_streams_state;
DROP INDEX IF EXISTS idx_streams_live;
DROP INDEX IF EXISTS idx_streams_starttime;
DROP INDEX IF EXISTS idx_streams_user_id;

/* Remove the "streams" table. */
DROP TABLE IF EXISTS streams;

/* Remove the "stream_state" type. */
DROP TYPE IF EXISTS stream_state;

-- ( 1 ) --
