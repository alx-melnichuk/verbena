use chrono::{DateTime, Utc};
use vrb_dbase::dbase::DbPool;

use crate::stream_models::{CreateStreamAndTags, ModifyStreamAndTags, SearchStreamAndTags, SearchStreamDate,
    SearchStreamTag, StreamAndTags, StreamTag};

pub trait StreamOrm {
    /// Add a new entry (stream, tags).
    fn create_stream_and_tags(&self, create_stream: CreateStreamAndTags) -> Result<Option<StreamAndTags>, String>;

    /// Modify an entity (stream, tags).
    fn modify_stream_and_tags(&self, id: i32, user_id: i32, modify_stream: ModifyStreamAndTags) -> Result<Option<StreamAndTags>, String>;

    /// Delete an entity (stream, tags).
    fn delete_stream_and_tags(&self, id: i32, user_id: i32) -> Result<Option<StreamAndTags>, String>;

    /// Get an entity from "streams" and "stream_tags" by ID.
    fn get_stream_and_tags(&self, id: i32) -> Result<Option<StreamAndTags>, String>;

    /// Get an entity with "live" from "streams" and "stream_tags" by user_id.
    fn get_stream_and_tags_in_live(&self, user_id: i32, exclude_id: i32) -> Result<Option<StreamAndTags>, String>;

    /// Filter entities (stream and tags) by the "SearchStream" search parameter.
    fn filter_stream_and_tags_by_pages(&self, search_stream: SearchStreamAndTags) -> Result<(u32, Vec<StreamAndTags>), String>;

    /// Filtering objects (stream dates) over a long interval (month) by parameters.
    fn filter_stream_dates(&self, search_stream: SearchStreamDate) -> Result<Vec<DateTime<Utc>>, String>;

    /// Get an entities (stream_tags) by the "SearchTag" search parameter.
    fn get_stream_tags(&self, search_stream: SearchStreamTag) -> Result<(u32, u32, Vec<StreamTag>), String>;
}

#[cfg(not(all(test, feature = "mockdata")))]
pub fn get_stream_orm_app(pool: DbPool) -> impls::StreamOrmApp {
    impls::StreamOrmApp::new(pool)
}
#[cfg(all(test, feature = "mockdata"))]
pub fn get_stream_orm_app(_: DbPool) -> tests::StreamOrmApp {
    tests::StreamOrmApp::new()
}

#[cfg(not(all(test, feature = "mockdata")))]
pub mod impls {
    use std::time::Instant as tm;

    use chrono::{DateTime, Utc};
    use diesel::{self, prelude::*, sql_types};
    use log::{Level::Info, info, log_enabled};
    use vrb_dbase::{dbase, schema};

    use crate::stream_models::{CountStreamAndTags, CreateStreamAndTags, ModifyStreamAndTags, SEARCH_STREAM_AND_TAGS_LIMIT,
        SEARCH_STREAM_AND_TAGS_PAGE, SEARCH_STREAM_TAGS_LIMIT, SEARCH_STREAM_TAGS_PAGE, SearchStreamAndTags, SearchStreamDate,
        SearchStreamTag, StartStreamAndTags, StreamAndTags, StreamTag};
    use crate::stream_orm::StreamOrm;

    pub const CONN_POOL: &str = "ConnectionPool";

    #[derive(Debug, Clone)]
    pub struct StreamOrmApp {
        pub pool: dbase::DbPool,
    }

    impl StreamOrmApp {
        pub fn new(pool: dbase::DbPool) -> Self {
            StreamOrmApp { pool }
        }
        pub fn get_conn(&self) -> Result<dbase::DbPooledConnection, String> {
            (&self.pool).get().map_err(|e| format!("{}: {}", CONN_POOL, e.to_string()))
        }
    }

    impl StreamOrm for StreamOrmApp {
        /// Add a new entry (stream, tags).
        fn create_stream_and_tags(&self, create_stream: CreateStreamAndTags) -> Result<Option<StreamAndTags>, String> {
            let timer = if log_enabled!(Info) { Some(tm::now()) } else { None };

            // Get a connection from the P2D2 pool.
            let mut conn = self.get_conn()?;
            #[rustfmt::skip]
            let query = diesel::sql_query(
                "select * from create_stream_and_stream_tag($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11);")
                .bind::<sql_types::Integer, _>(create_stream.user_id) // $1
                .bind::<sql_types::Text, _>(create_stream.title) // $2
                .bind::<sql_types::Nullable<sql_types::Text>, _>(create_stream.descript) // $3
                .bind::<sql_types::Nullable<sql_types::Text>, _>(create_stream.logo) // $4
                .bind::<sql_types::Timestamptz, _>(create_stream.starttime) // $5
                .bind::<sql_types::Nullable<schema::sql_types::StreamState>, _>(create_stream.state) // $6
                .bind::<sql_types::Nullable<sql_types::Timestamptz>, _>(create_stream.started) // $7
                .bind::<sql_types::Nullable<sql_types::Timestamptz>, _>(create_stream.paused) // $8
                .bind::<sql_types::Nullable<sql_types::Timestamptz>, _>(create_stream.stopped) // $9
                .bind::<sql_types::Nullable<sql_types::Text>, _>(create_stream.source) // $10
                .bind::<sql_types::Array<sql_types::Text>, _>(create_stream.tags); // $11

            // Run a query with Diesel to create a new user and return it.
            let opt_stream_and_tags = query
                .get_result::<StreamAndTags>(&mut conn)
                .optional()
                .map_err(|e| format!("create_stream_and_stream_tag: {}", e.to_string()))?;

            if let Some(timer) = timer {
                info!("create_stream_and_tags() time: {}", format!("{:.2?}", timer.elapsed()));
            }
            Ok(opt_stream_and_tags)
        }

        /// Modify an entity (stream, tags).
        #[rustfmt::skip]
        fn modify_stream_and_tags(
            &self, id: i32, user_id: i32, modify_stream: ModifyStreamAndTags,
        ) -> Result<Option<StreamAndTags>, String> {
            let timer = if log_enabled!(Info) { Some(tm::now()) } else { None };

            // Get a connection from the P2D2 pool.
            let mut conn = self.get_conn()?;
            #[rustfmt::skip]
            let query = diesel::sql_query(
                "select * from modify_stream_and_stream_tag($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12);")
                .bind::<sql_types::Integer, _>(id) // $1
                .bind::<sql_types::Integer, _>(user_id) // $2
                .bind::<sql_types::Nullable<sql_types::Text>, _>(modify_stream.title) // $3
                .bind::<sql_types::Nullable<sql_types::Text>, _>(modify_stream.descript) // $4
                .bind::<sql_types::Nullable<sql_types::Text>, _>(modify_stream.logo) // $5
                .bind::<sql_types::Nullable<sql_types::Timestamptz>, _>(modify_stream.starttime) // $6
                .bind::<sql_types::Nullable<schema::sql_types::StreamState>, _>(modify_stream.state) // $7
                .bind::<sql_types::Nullable<sql_types::Timestamptz>, _>(modify_stream.started) // $8
                .bind::<sql_types::Nullable<sql_types::Timestamptz>, _>(modify_stream.paused) // $9
                .bind::<sql_types::Nullable<sql_types::Timestamptz>, _>(modify_stream.stopped) // $10
                .bind::<sql_types::Nullable<sql_types::Text>, _>(modify_stream.source) // $11
                .bind::<sql_types::Nullable<sql_types::Array<sql_types::Text>>, _>(modify_stream.tags); // $12
                
            // Run a query with Diesel to create a new user and return it.
            let opt_stream_and_tags = query
                .get_result::<StreamAndTags>(&mut conn)
                .optional()
                .map_err(|e| format!("modify_stream_and_stream_tag: {}", e.to_string()))?;

            if let Some(timer) = timer {
                info!("modify_stream_and_stream_tag() time: {}", format!("{:.2?}", timer.elapsed()));
            }
            Ok(opt_stream_and_tags)
        }

        /// Delete an entity (stream, tags).
        fn delete_stream_and_tags(&self, id: i32, user_id: i32) -> Result<Option<StreamAndTags>, String> {
            let timer = if log_enabled!(Info) { Some(tm::now()) } else { None };
            // Get a connection from the P2D2 pool.
            let mut conn = self.get_conn()?;

            let query = diesel::sql_query(
                "select * from delete_stream_and_stream_tag($1,$2);")
                .bind::<sql_types::Integer, _>(id) // $1
                .bind::<sql_types::Integer, _>(user_id); // $2

            // Run a query using Diesel and get the result.
            let opt_stream_and_tags = query
                .get_result::<StreamAndTags>(&mut conn)
                .optional()
                .map_err(|e| format!("delete_stream_and_tags: {}", e.to_string()))?;

            if let Some(timer) = timer {
                info!("delete_stream_and_tags() time: {}", format!("{:.2?}", timer.elapsed()));
            }
            Ok(opt_stream_and_tags)
        }

        /// Get an entity from "streams" and "stream_tags" by ID.
        fn get_stream_and_tags(&self, id: i32) -> Result<Option<StreamAndTags>, String> {
            let timer: Option<tm> = if log_enabled!(Info) { Some(tm::now()) } else { None };
            // Get a connection from the P2D2 pool.
            let mut conn = self.get_conn()?;

            let query = diesel::sql_query(
                "select * from get_stream_and_stream_tag($1);")
                .bind::<sql_types::Integer, _>(id); // $1

            // Run a query using Diesel and get the result.
            let opt_stream_and_tags = query
                .get_result::<StreamAndTags>(&mut conn)
                .optional()
                .map_err(|e| format!("get_stream_and_stream_tag: {}", e.to_string()))?;

            if let Some(timer) = timer {
                info!("get_stream_and_tags() time: {}", format!("{:.2?}", timer.elapsed()));
            }
            Ok(opt_stream_and_tags)
        }

        /// Get an entity with "live" from "streams" and "stream_tags" by user_id.
        fn get_stream_and_tags_in_live(&self, user_id: i32, exclude_id: i32) -> Result<Option<StreamAndTags>, String> {
            let timer = if log_enabled!(Info) { Some(tm::now()) } else { None };
            // Get a connection from the P2D2 pool.
            let mut conn = self.get_conn()?;

            let query = diesel::sql_query(
                "SELECT s.*, ARRAY[]::VARCHAR[] AS tags, NULL AS old_logo \
                 FROM streams s, \
                 filter_stream_and_stream_tag_by_pages_ids($1,true,null,null,null,null,null,2,null) f \
                 WHERE s.id = f.id AND s.id != $2 \
                 LIMIT 1")
                .bind::<sql_types::Integer, _>(user_id) // $1
                .bind::<sql_types::Integer, _>(exclude_id); // $2

            // Run a query using Diesel and get the result.
            let opt_stream_and_tags = query
                .get_result::<StreamAndTags>(&mut conn)
                .optional()
                .map_err(|e| format!("(1)filter_stream_and_stream_tag_by_pages_ids: {}", e.to_string()))?;

            if let Some(timer) = timer {
                info!("get_stream_and_tags_in_live() time: {}", format!("{:.2?}", timer.elapsed()));
            }
            Ok(opt_stream_and_tags)
        }

        /// Filter entities (stream and tags) by the "SearchStream" search parameter.
        fn filter_stream_and_tags_by_pages(&self, search_stream: SearchStreamAndTags) -> Result<(u32, Vec<StreamAndTags>), String> {
            let timer = if log_enabled!(Info) { Some(tm::now()) } else { None };
            // Get a connection from the P2D2 pool.
            let mut conn = self.get_conn()?;

            let page: i32 = search_stream.page.unwrap_or(SEARCH_STREAM_AND_TAGS_PAGE).try_into().unwrap();
            let limit: i32 = search_stream.limit.unwrap_or(SEARCH_STREAM_AND_TAGS_LIMIT).try_into().unwrap();
            let offset: i32 = (page - 1) * limit;
            let filter = search_stream.filter.clone().map(|v| v.to_string());
            let search_stream2 = search_stream.clone();
            let filter2 = filter.clone();

            #[rustfmt::skip]
            let query = diesel::sql_query(
                "SELECT * FROM filter_stream_and_stream_tag_by_pages_list($1,$2,$3,$4,$5,$6,$7,$8,$9);")
                .bind::<sql_types::Nullable<sql_types::Integer>, _>(search_stream.user_id) // $1
                .bind::<sql_types::Nullable<sql_types::Bool>, _>(search_stream.live) // $2
                .bind::<sql_types::Nullable<sql_types::Text>, _>(filter) // $3
                .bind::<sql_types::Nullable<sql_types::Timestamptz>, _>(search_stream.starttime) // $4
                .bind::<sql_types::Nullable<sql_types::Timestamptz>, _>(search_stream.finishtime) // $5
                .bind::<sql_types::Nullable<sql_types::Bool>, _>(search_stream.sort_desc) // $6
                .bind::<sql_types::Nullable<sql_types::Text>, _>(search_stream.tag) // $7
                .bind::<sql_types::Integer, _>(limit) // $8
                .bind::<sql_types::Integer, _>(offset); // $9

            // Run a query using Diesel and get the result.
            let stream_and_tags_list: Vec<StreamAndTags> = query
                //  .get_results::<Vec<StreamAndTags>>(&mut conn)
                .load(&mut conn)
                .map_err(|e| format!("filter_stream_and_stream_tag_by_pages_list: {}", e.to_string()))?;

            #[rustfmt::skip]
            let query = diesel::sql_query(
                "SELECT * FROM filter_stream_and_stream_tag_by_pages_count($1,$2,$3,$4,$5,$6) as cnt;")
                .bind::<sql_types::Nullable<sql_types::Integer>, _>(search_stream2.user_id) // $1
                .bind::<sql_types::Nullable<sql_types::Bool>, _>(search_stream2.live) // $2
                .bind::<sql_types::Nullable<sql_types::Text>, _>(filter2) // $3
                .bind::<sql_types::Nullable<sql_types::Timestamptz>, _>(search_stream2.starttime) // $4
                .bind::<sql_types::Nullable<sql_types::Timestamptz>, _>(search_stream2.finishtime) // $5
                .bind::<sql_types::Nullable<sql_types::Text>, _>(search_stream2.tag); // $6

            // Run a query using Diesel and get the result.
            let count_stream_and_tags = query
                .get_result::<CountStreamAndTags>(&mut conn)
                .map_err(|e| format!("filter_stream_and_stream_tag_by_pages_count: {}", e.to_string()))?;

            let count: u32 = count_stream_and_tags.cnt.try_into().unwrap();

            if let Some(timer) = timer {
                info!("filter_stream_and_tags_by_pages() time: {}", format!("{:.2?}", timer.elapsed()));
            }
            Ok((count, stream_and_tags_list))
        }

        /// Filtering objects (stream dates) over a long interval (month) by parameters.
        fn filter_stream_dates(&self, search_stream: SearchStreamDate) -> Result<Vec<DateTime<Utc>>, String> {
            let timer = if log_enabled!(Info) { Some(tm::now()) } else { None };

            // Get a connection from the P2D2 pool.
            let mut conn = self.get_conn()?;

            #[rustfmt::skip]
            let query = diesel::sql_query(
                "SELECT s.starttime AS start \
                 FROM streams s, \
                   filter_stream_and_stream_tag_by_pages_ids($1,null,'period',$2,$3,null,null,null,null) f \
                 WHERE s.id = f.id;")
                .bind::<sql_types::Integer, _>(search_stream.user_id) // $1
                .bind::<sql_types::Timestamptz, _>(search_stream.start) // $2
                .bind::<sql_types::Timestamptz, _>(search_stream.finish); // $3

            // Run a query using Diesel and get the result.
            let stream_date_list: Vec<StartStreamAndTags> = query
                //  .get_results::<Vec<StartStreamAndTags>>(&mut conn)
                .load(&mut conn)
                .map_err(|e| format!("(3)filter_stream_and_stream_tag_by_pages_ids: {}", e.to_string()))?;
            
            let list = stream_date_list.into_iter().into_iter().map(|v| v.start).collect();
            // let list: Vec<DateTime<Utc> = stream_date_list.into_iter().map(|v| v.start).collect();
            
            if let Some(timer) = timer {
                info!("filter_stream_dates() time: {}", format!("{:.2?}", timer.elapsed()));
            }
            Ok(list)
        }

        /// Get an entities (stream_tags) by the "SearchTag" search parameter.
        fn get_stream_tags(&self, search_stream: SearchStreamTag) -> Result<(u32, u32, Vec<StreamTag>), String> {
            let timer: Option<tm> = if log_enabled!(Info) { Some(tm::now()) } else { None };
            // Get a connection from the P2D2 pool.
            let mut conn = self.get_conn()?;

            let sort_column = search_stream.sort_column.clone().map(|v| v.to_string());
            let page: u32 = search_stream.page.unwrap_or(SEARCH_STREAM_TAGS_PAGE);
            let limit: u32 = search_stream.limit.unwrap_or(SEARCH_STREAM_TAGS_LIMIT);
            let offset: u32 = (page - 1) * limit;
            let limit2: i32 = limit.try_into().unwrap();
            let offset2: i32 = offset.try_into().unwrap();
            #[rustfmt::skip]
            let query = diesel::sql_query(
                "SELECT * FROM get_stream_tags($1,$2,$3,$4);")
                .bind::<sql_types::Nullable<sql_types::Text>, _>(sort_column) // $1
                .bind::<sql_types::Nullable<sql_types::Bool>, _>(search_stream.sort_desc) // $2
                .bind::<sql_types::Integer, _>(limit2) // $3
                .bind::<sql_types::Integer, _>(offset2); // $4

            // Run a query using Diesel and get the result.
            let stream_tag_list: Vec<StreamTag> = query
                //  .get_results::<Vec<StreamTag>>(&mut conn)
                .load(&mut conn)
                .map_err(|e| format!("get_stream_tags: {}", e.to_string()))?;

            if let Some(timer) = timer {
                info!("get_stream_tags() time: {}", format!("{:.2?}", timer.elapsed()));
            }
            Ok((limit, page, stream_tag_list))

        }

    }
}

// ** **

#[cfg(all(test, feature = "mockdata"))]
pub mod tests {
    use std::cmp::Ordering;

    use actix_web::web;
    use chrono::{DateTime, Utc};
    use vrb_authent::user_orm::tests::{USER_IDS, USER1_ID};

    use crate::config_strm;
    use crate::stream_models::{
        CreateStreamAndTags, FilterStream, ModifyStreamAndTags, SEARCH_STREAM_AND_TAGS_LIMIT, SEARCH_STREAM_AND_TAGS_PAGE,
        SEARCH_STREAM_TAGS_LIMIT, SEARCH_STREAM_TAGS_PAGE, SearchStreamAndTags, SearchStreamDate, SearchStreamTag, StreamAndTags,
        StreamTag, StreamTagSortColumn};
    use crate::stream_orm::StreamOrm;

    pub const STREAM_ID: i32 = 1400;
    pub const STREAM_TAG_ID: i32 = 1500;
    pub const TAG_NAME: &str = "tag";

    #[derive(Debug, Clone)]
    pub struct StreamOrmApp {
        pub stream_info_vec: Vec<StreamAndTags>,
    }

    impl StreamOrmApp {
        /// Create a new instance.
        pub fn new() -> Self {
            StreamOrmApp {
                stream_info_vec: Vec::new(),
            }
        }
        /// Create a new instance with the specified `stream` list.
        #[cfg(test)]
        pub fn create(stream_vec: &[StreamAndTags]) -> Self {
            let mut stream_info_vec: Vec<StreamAndTags> = Vec::new();

            for (idx, stream) in stream_vec.iter().enumerate() {
                use vrb_dbase::enm_stream_state::StreamState;

                let mut stream2 = stream.clone();
                stream2.live = StreamState::is_live(stream.state);
                let delta: i32 = idx.try_into().unwrap();
                stream2.id = STREAM_ID + delta;
                stream_info_vec.push(stream2);
            }
            StreamOrmApp { stream_info_vec }
        }
    }

    impl StreamOrm for StreamOrmApp {
        /// Add a new entry (stream, tags).
        fn create_stream_and_tags(&self, create_stream: CreateStreamAndTags) -> Result<Option<StreamAndTags>, String> {
            let len: i32 = self.stream_info_vec.len().try_into().unwrap(); // convert usize as i32
            
            let mut stream: StreamAndTags = create_stream.into();
            stream.id = STREAM_ID + len;

            Ok(Some(stream))
        }
        /// Modify an entity (stream, tags).
        fn modify_stream_and_tags(
            &self, id: i32, user_id: i32, modify_stream: ModifyStreamAndTags
        ) -> Result<Option<StreamAndTags>, String> {
            let opt_stream_info = self.stream_info_vec
                .iter()
                .find(|stream| stream.id == id && stream.user_id == user_id)
                .map(|stream| stream.clone());
            
            if let Some(stream_info) = opt_stream_info {
                let stream = modify_stream.merge(stream_info);
                Ok(Some(stream))
            } else {
                Ok(None)
            }
        }
        /// Delete an entity (stream, tags).
        fn delete_stream_and_tags(&self, id: i32, user_id: i32) -> Result<Option<StreamAndTags>, String> {
            let opt_stream_info = self.stream_info_vec
                .iter()
                .find(|stream| stream.id == id && stream.user_id == user_id)
                .map(|stream| stream.clone());

            Ok(opt_stream_info)
        }
        /// Get an entity from "streams" and "stream_tags" by ID.
        fn get_stream_and_tags(&self, id: i32) -> Result<Option<StreamAndTags>, String> {
            let opt_stream_info = self.stream_info_vec
                .iter()
                .find(|stream| stream.id == id)
                .map(|stream| stream.clone());

            Ok(opt_stream_info)
        }
        /// Get an entity with "live" from "streams" and "stream_tags" by user_id.
        fn get_stream_and_tags_in_live(&self, user_id: i32, exclude_id: i32) -> Result<Option<StreamAndTags>, String> {
            let opt_stream_info = self.stream_info_vec
                .iter()
                .find(|stream| stream.live == true && stream.user_id == user_id && stream.id != exclude_id)
                .map(|stream| stream.clone());
            
            if let Some(stream_info) = opt_stream_info {
                let mut stream = stream_info.clone();
                stream.tags = vec![];
                Ok(Some(stream))
            } else {
                Ok(None)
            }
        }
        /// Filter entities (stream and tags) by the "SearchStream" search parameter.
        fn filter_stream_and_tags_by_pages(&self, search: SearchStreamAndTags) -> Result<(u32, Vec<StreamAndTags>), String> {
            let mut streams_info: Vec<StreamAndTags> = vec![];

            let future_start = if let Some(FilterStream::Future) = search.filter { search.starttime } else { None }; 
            let past_start = if let Some(FilterStream::Past) = search.filter { search.starttime } else { None }; 
            let period_start = if let Some(FilterStream::Period) = search.filter { search.starttime } else { None }; 
            let period_finish = if let Some(FilterStream::Period) = search.filter { search.finishtime } else { None };
            let search_tag = search.tag.unwrap_or_default();

            for stream in self.stream_info_vec.iter() {
                if search.user_id.is_some() && stream.user_id != search.user_id.unwrap() {
                    continue;
                }
                if search.live.is_some() && stream.live != search.live.unwrap() {
                    continue;
                }
                if future_start.is_some() {
                    // future_start <= stream.starttime
                    if stream.starttime < future_start.unwrap() {
                        continue;
                    }
                } else if past_start.is_some() {
                    // past_start < stream.starttime
                    if stream.starttime >= past_start.unwrap() {
                        continue;
                    }
                } else if period_start.is_some() && period_finish.is_some() {
                    // period_start <= stream.starttime <= period_finish
                    if stream.starttime < period_start.unwrap() || period_finish.unwrap() < stream.starttime {
                        continue;
                    }
                }
                if search_tag.len() > 0 && !stream.tags.contains(&search_tag) {
                    continue;
                }

                streams_info.push(stream.clone());
            }

            let is_starttime_desc = search.sort_desc.unwrap_or(false);

            streams_info.sort_by(|a, b| {
                let mut result = if !is_starttime_desc {
                    a.starttime.partial_cmp(&b.starttime).unwrap_or(Ordering::Equal)
                } else {
                    b.starttime.partial_cmp(&a.starttime).unwrap_or(Ordering::Equal)
                };
                if result == Ordering::Equal {
                    result = a.id.partial_cmp(&b.id).unwrap_or(Ordering::Equal);
                }
                
                result
            });

            let amount = streams_info.len();
            let page = search.page.unwrap_or(SEARCH_STREAM_AND_TAGS_PAGE);
            let limit = search.limit.unwrap_or(SEARCH_STREAM_AND_TAGS_LIMIT);
            let min_idx = (page - 1) * limit;
            let max_idx = min_idx + limit;
            let mut idx = 0;
            streams_info.retain(|_| {
                let res = min_idx <= idx && idx < max_idx;
                idx += 1;
                res
            });

            let count: u32 = amount.try_into().unwrap();
            let streams: Vec<StreamAndTags> = streams_info.iter().map(|v| v.clone()).collect();

            Ok((count, streams))
        }
        /// Filtering objects (stream dates) over a long interval (month) by parameters.
        fn filter_stream_dates(&self, search_stream: SearchStreamDate) -> Result<Vec<DateTime<Utc>>, String> {
            
            let search = SearchStreamAndTags::new(
                Some(search_stream.user_id),
                Some(FilterStream::Period), 
                Some(search_stream.start),
                Some(search_stream.finish)
            );
            
            let res = self.filter_stream_and_tags_by_pages(search);

            let result: Vec<DateTime<Utc>> = if let Ok((_count, streams)) = res {
                streams.into_iter().map(|v| v.starttime).collect()
            } else {
                vec![]
            };

            Ok(result)
        }
        /// Get an entities (stream_tags) by the "SearchTag" search parameter.
        fn get_stream_tags(&self, search_stream: SearchStreamTag) -> Result<(u32, u32, Vec<StreamTag>), String> {
            let mut strm_tags: Vec<StreamTag> = vec![StreamTag { id: STREAM_TAG_ID, name: TAG_NAME.into(), count_links: 0 }];

            let delta = STREAM_TAG_ID - USER1_ID + 1;
            for stream in self.stream_info_vec.iter() {
                for tag_str in stream.tags.iter() {
                    let tag_name = tag_str.clone();
                    let opt_strm_tag = strm_tags.iter_mut().find(|t| t.name == tag_name);
                    if let Some(strm_tag) = opt_strm_tag {
                        strm_tag.count_links += 1;
                    } else {
                        strm_tags.push(StreamTag { id: stream.user_id + delta, name: tag_name, count_links: 1 });
                    }
                }
            }

            let sort_desc = search_stream.sort_desc.unwrap_or_default();
            strm_tags.sort_by(|a, b| {
                let val1 = if sort_desc { b } else { a };
                let val2 = if sort_desc { a } else { b };
                let result = match search_stream.sort_column {
                    Some(StreamTagSortColumn::Name) => 
                        val1.name.partial_cmp(&val2.name).unwrap_or(Ordering::Equal),
                    Some(StreamTagSortColumn::CountLinks) => 
                        val1.count_links.partial_cmp(&val2.count_links).unwrap_or(Ordering::Equal),
                    _ =>
                        val1.id.partial_cmp(&val2.id).unwrap_or(Ordering::Equal),
                };
                result
            });

            let page = search_stream.page.unwrap_or(SEARCH_STREAM_TAGS_PAGE);
            let limit = search_stream.limit.unwrap_or(SEARCH_STREAM_TAGS_LIMIT);
            let min_idx = (page - 1) * limit;
            let max_idx = min_idx + limit;
            let mut idx = 0;
            strm_tags.retain(|_| {
                let res = min_idx <= idx && idx < max_idx;
                idx += 1;
                res
            });

            Ok((limit, page, strm_tags))
        }
    }

    pub struct StreamOrmTest {}

    impl StreamOrmTest {
        pub fn stream_ids() -> Vec<i32> {
            vec![
                1, // Owner user idx 0 (live: true)  1100 oliver_taylor
                2, // Owner user idx 1 (live: true)  1101 robert_brown
                3, // Owner user idx 2 (live: false) 1102 mary_williams
                4, // Owner user idx 3  blocked      1103 ava_wilson
            ]
        }
        pub fn create_stream(idx: u8, user_id: i32, title: &str, tags: &str, starttime: DateTime<Utc>) -> StreamAndTags {
            let tags1: Vec<String> = tags.split(',').map(|val| val.to_string()).collect();
            let stream = StreamAndTags::new(STREAM_ID + i32::from(idx), user_id, title, starttime, &tags1);
            stream
        }
        pub fn streams(user_idxs: &[usize]) -> Vec<StreamAndTags> {
            let mut stream_info_vec: Vec<StreamAndTags> = Vec::new();
            let user_ids = USER_IDS.clone();
            for (index, user_idx) in user_idxs.iter().enumerate() {
                if let Some(user_id) = user_ids.get(*user_idx) {
                    let title = format!("title_{}_{}", index, *user_idx);
                    let tags = format!("{},{}{}", TAG_NAME, TAG_NAME, *user_idx + 1);
                    let idx = u8::try_from(index).unwrap();
                    let stream_info = Self::create_stream(idx, *user_id, &title, &tags, Utc::now());
                    stream_info_vec.push(stream_info);
                }
            }
            stream_info_vec
        }
        pub fn cfg_config_strm(config_strm: config_strm::ConfigStrm) -> impl FnOnce(&mut web::ServiceConfig) {
            move |config: &mut web::ServiceConfig| {
                let data_config_strm = web::Data::new(config_strm);
                config.app_data(web::Data::clone(&data_config_strm));
            }
        }
        pub fn cfg_stream_orm(data_s: Vec<StreamAndTags>) -> impl FnOnce(&mut web::ServiceConfig) {
            move |config: &mut web::ServiceConfig| {
                let data_stream_orm = web::Data::new(StreamOrmApp::create(&data_s));
                config.app_data(web::Data::clone(&data_stream_orm));
            }
        }
    }
}