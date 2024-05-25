use chrono::{DateTime, Utc};

use super::stream_models::{
    CreateStream, ModifyStream, SearchStream, SearchStreamEvent, SearchStreamPeriod, Stream, StreamTagStreamId,
};

pub trait StreamOrm {
    /// Find for an entity (stream) by id.
    fn find_stream_by_id(
        &self,
        id: i32,
        opt_user_id: Option<i32>,
    ) -> Result<Option<(Stream, Vec<StreamTagStreamId>)>, String>;
    /// Find for an entity (stream) by SearchStreamInfo.
    fn find_streams(&self, search_stream: SearchStream) -> Result<(u32, Vec<Stream>, Vec<StreamTagStreamId>), String>;
    /// Find for an entity (stream event) by SearchStreamEvent.
    fn find_stream_events(&self, search_stream_event: SearchStreamEvent) -> Result<(u32, Vec<Stream>), String>;
    /// Find for an entity (stream period) by SearchStreamPeriod.
    fn find_streams_period(&self, search_stream_period: SearchStreamPeriod) -> Result<Vec<DateTime<Utc>>, String>;
    /// Add a new entity (stream).
    fn create_stream(
        &self,
        create_stream: CreateStream,
        tags: &[String],
    ) -> Result<(Stream, Vec<StreamTagStreamId>), String>;
    /// Get the logo file name for an entity (stream) by ID.
    fn get_stream_logo_by_id(&self, id: i32) -> Result<Option<String>, String>;
    /// Modify an entity (stream).
    fn modify_stream(
        &self,
        id: i32,
        opt_user_id: Option<i32>,
        modify_stream: ModifyStream,
        opt_tags: Option<Vec<String>>,
    ) -> Result<Option<(Stream, Vec<StreamTagStreamId>)>, String>;
    /// Delete an entity (stream).
    fn delete_stream(
        &self,
        id: i32,
        opt_user_id: Option<i32>,
    ) -> Result<Option<(Stream, Vec<StreamTagStreamId>)>, String>;
}

pub mod cfg {
    use crate::dbase::DbPool;

    #[cfg(not(feature = "mockdata"))]
    use super::inst::StreamOrmApp;
    #[cfg(not(feature = "mockdata"))]
    pub fn get_stream_orm_app(pool: DbPool) -> StreamOrmApp {
        StreamOrmApp::new(pool)
    }

    #[cfg(feature = "mockdata")]
    use super::tests::StreamOrmApp;
    #[cfg(feature = "mockdata")]
    pub fn get_stream_orm_app(_: DbPool) -> StreamOrmApp {
        StreamOrmApp::new()
    }
}

#[cfg(not(feature = "mockdata"))]
pub mod inst {
    use chrono::Duration;
    use diesel::{self, prelude::*, sql_types};
    use schema::streams::dsl as streams_dsl;

    use crate::dbase;
    use crate::schema;
    use crate::streams::stream_models::{self, CreateStream, SearchStreamPeriod};

    use super::*;

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
        /// Get a list of "tags" for the specified "stream".
        fn get_stream_tags(
            &self,
            conn: &mut dbase::DbPooledConnection,
            ids: &[i32],
        ) -> Result<Vec<StreamTagStreamId>, diesel::result::Error> {
            let query = diesel::sql_query("select * from get_stream_tags_names($1);")
                .bind::<sql_types::Array<sql_types::Integer>, _>(ids);

            query.get_results::<StreamTagStreamId>(conn)
        }
        /// Update the list of "tags" for the specified "stream".
        fn update_list_stream_tags(
            &self,
            conn: &mut dbase::DbPooledConnection,
            stream_id: i32,
            user_id: i32,
            tags: &[String],
        ) -> Result<usize, diesel::result::Error> {
            let stream_tags: Vec<String> = tags.iter().map(|tag| tag.to_lowercase().trim().to_string()).collect();
            // Run query using Diesel to add a list of "stream_tags" for the entity (stream).
            let query = diesel::sql_query("CALL update_list_stream_tags($1, $2, $3);")
                .bind::<sql_types::Integer, _>(stream_id)
                .bind::<sql_types::Integer, _>(user_id)
                .bind::<sql_types::Array<sql_types::Text>, _>(stream_tags);

            query.execute(conn)
        }
        /// Update the "stream_tags" data for user.
        fn update_stream_tags_for_user(
            &self,
            conn: &mut dbase::DbPooledConnection,
            user_id: i32,
        ) -> Result<(), diesel::result::Error> {
            // Run query using Diesel to update the list of "stream_tags" for the user.
            let query =
                diesel::sql_query("CALL update_stream_tags_for_user($1);").bind::<sql_types::Integer, _>(user_id);

            query.execute(conn)?;
            Ok(())
        }
    }

    impl StreamOrm for StreamOrmApp {
        /// Find for an entity (stream) by id.
        fn find_stream_by_id(
            &self,
            id: i32,
            opt_user_id: Option<i32>,
        ) -> Result<Option<(Stream, Vec<StreamTagStreamId>)>, String> {
            // Get a connection from the P2D2 pool.
            let mut conn = self.get_conn()?;

            let mut query_list = schema::streams::table.into_boxed();
            // Find stream by id.
            query_list = query_list.filter(streams_dsl::id.eq(id));

            if let Some(user_id) = opt_user_id {
                // Find stream by id and user_id.
                query_list = query_list.filter(streams_dsl::user_id.eq(user_id));
            }
            // Run query using Diesel to find user by id (and user_id) and return it.
            let opt_stream = query_list
                .first::<Stream>(&mut conn)
                .optional()
                .map_err(|e| format!("find_stream_by_id: {}", e.to_string()))?;

            if let Some(stream) = opt_stream {
                let stream_tags = self
                    .get_stream_tags(&mut conn, &[stream.id])
                    .map_err(|e| format!("get_stream_tags: {}", e.to_string()))?;
                Ok(Some((stream, stream_tags)))
            } else {
                Ok(None)
            }
        }
        /// Find for an entity (stream) by SearchStream.
        fn find_streams(
            &self,
            search_stream: SearchStream,
        ) -> Result<(u32, Vec<Stream>, Vec<StreamTagStreamId>), String> {
            // Get a connection from the P2D2 pool.
            let mut conn = self.get_conn()?;

            let page: u32 = search_stream.page.unwrap_or(stream_models::SEARCH_STREAM_PAGE);
            let limit: u32 = search_stream.limit.unwrap_or(stream_models::SEARCH_STREAM_LIMIT);
            let offset: u32 = (page - 1) * limit;

            let order_column = search_stream.order_column.unwrap_or(stream_models::SEARCH_STREAM_ORDER_COLUMN);
            let order_direction = search_stream
                .order_direction
                .unwrap_or(stream_models::SEARCH_STREAM_ORDER_DIRECTION);
            let is_asc = order_direction == stream_models::OrderDirection::Asc;

            // Build a query to find a list of "streams".
            let mut query_list = schema::streams::table.into_boxed();
            query_list = query_list
                .select(schema::streams::all_columns)
                .filter(streams_dsl::user_id.eq(search_stream.user_id))
                .offset(offset.into())
                .limit(limit.into());

            // Create a query to get the number of elements in the list of "streams".
            let mut query_count = schema::streams::table.into_boxed();
            query_count = query_count.filter(streams_dsl::user_id.eq(search_stream.user_id));

            if let Some(live) = search_stream.live {
                query_list = query_list.filter(streams_dsl::live.eq(live));
                query_count = query_count.filter(streams_dsl::live.eq(live));
            }

            if let Some(is_future) = search_stream.is_future {
                let now_date = Utc::now().date_naive().and_hms_opt(0, 0, 0).unwrap();
                if !is_future {
                    // starttime < now_date
                    query_list = query_list.filter(streams_dsl::starttime.lt(now_date));
                    query_count = query_count.filter(streams_dsl::starttime.lt(now_date));
                } else {
                    // starttime >= now_date
                    query_list = query_list.filter(streams_dsl::starttime.ge(now_date));
                    query_count = query_count.filter(streams_dsl::starttime.ge(now_date));
                }
            }

            if order_column == stream_models::OrderColumn::Starttime {
                if is_asc {
                    query_list = query_list.order_by(streams_dsl::starttime.asc());
                } else {
                    query_list = query_list.order_by(streams_dsl::starttime.desc());
                }
            } else {
                if is_asc {
                    query_list = query_list.order_by(streams_dsl::title.asc());
                } else {
                    query_list = query_list.order_by(streams_dsl::title.desc());
                }
            }
            query_list = query_list.then_order_by(streams_dsl::id.asc());

            let amount_res = query_count.count().get_result::<i64>(&mut conn);
            // lead time: 476.06µs
            if let Err(err) = amount_res {
                return Err(format!("find_streams: (query_count) {}", err));
            }
            let amount: i64 = amount_res.unwrap();
            let count: u32 = amount.try_into().unwrap();

            let streams: Vec<Stream> = query_list
                .load(&mut conn)
                .map_err(|e| format!("find_streams: (query_list) {}", e.to_string()))?;
            // lead time: 679.46µs
            // Get a list of "stream" identifiers.
            let ids: Vec<i32> = streams.iter().map(|stream| stream.id).collect();
            let stream_tags = self
                .get_stream_tags(&mut conn, &ids)
                .map_err(|e| format!("get_stream_tags: {}", e.to_string()))?;

            Ok((count, streams, stream_tags))
        }

        /// Find for an entity (stream event) by SearchStreamEvent.
        fn find_stream_events(&self, search_event: SearchStreamEvent) -> Result<(u32, Vec<Stream>), String> {
            // Get a connection from the P2D2 pool.
            let mut conn = self.get_conn()?;

            let page: u32 = search_event.page.unwrap_or(stream_models::SEARCH_STREAM_EVENT_PAGE);
            let limit: u32 = search_event.limit.unwrap_or(stream_models::SEARCH_STREAM_EVENT_LIMIT);
            let offset: u32 = (page - 1) * limit;
            let start = search_event.starttime;
            let finish = start + Duration::hours(24);

            // Build a query to find a list of "streams".
            let mut query_list = schema::streams::table.into_boxed();
            query_list = query_list
                .select(schema::streams::all_columns)
                // starttime >= start
                .filter(streams_dsl::starttime.ge(start))
                // starttime < finish
                .filter(streams_dsl::starttime.lt(finish))
                .filter(streams_dsl::user_id.eq(search_event.user_id))
                .offset(offset.into())
                .limit(limit.into());

            // Create a query to get the number of elements in the list of "streams".
            let mut query_count = schema::streams::table.into_boxed();
            query_count = query_count
                // starttime >= start
                .filter(streams_dsl::starttime.ge(start))
                // starttime < finish
                .filter(streams_dsl::starttime.lt(finish))
                .filter(streams_dsl::user_id.eq(search_event.user_id));

            query_list = query_list
                .order_by(streams_dsl::starttime.asc())
                .then_order_by(streams_dsl::id.asc());

            let amount_res = query_count.count().get_result::<i64>(&mut conn);
            // lead time: 1.14ms
            if let Err(err) = amount_res {
                return Err(format!("find_stream_events: (query_count) {}", err));
            }
            let amount: i64 = amount_res.unwrap();
            let count: u32 = amount.try_into().unwrap();

            let streams: Vec<Stream> = query_list
                .load(&mut conn)
                .map_err(|e| format!("find_streams: (query_list) {}", e.to_string()))?;
            // lead time: 699.49µs

            // lead time: 2.14ms
            Ok((count, streams))
        }

        /// Find for an entity (stream period) by SearchStreamPeriod.
        fn find_streams_period(&self, search_period: SearchStreamPeriod) -> Result<Vec<DateTime<Utc>>, String> {
            // Get a connection from the P2D2 pool.
            let mut conn = self.get_conn()?;

            let start = search_period.start;
            let finish = search_period.finish;
            // Build a query to find a list of "streams"
            let query_list = schema::streams::table
                .select(schema::streams::columns::starttime)
                // starttime >= start
                .filter(streams_dsl::starttime.ge(start))
                // starttime <= finish
                .filter(streams_dsl::starttime.le(finish))
                .filter(streams_dsl::user_id.eq(search_period.user_id))
                .order_by(streams_dsl::starttime.asc())
                .then_order_by(streams_dsl::id.asc());

            let list: Vec<DateTime<Utc>> = query_list
                .load(&mut conn)
                .map_err(|e| format!("find_streams_period: {}", e.to_string()))?;
            // lead time: 704.62µs

            // lead time: 908.45µs
            Ok(list)
        }

        /// Add a new entity (stream).
        fn create_stream(
            &self,
            create_stream: CreateStream,
            tags: &[String],
        ) -> Result<(Stream, Vec<StreamTagStreamId>), String> {
            // Get a connection from the P2D2 pool.
            let mut conn = self.get_conn()?;
            let mut err_table = "create_stream";

            let res_data = conn.transaction::<_, diesel::result::Error, _>(|conn| {
                // Run query using Diesel to add a new entry (stream).
                let res_stream = diesel::insert_into(schema::streams::table)
                    .values(create_stream)
                    .returning(Stream::as_returning())
                    .get_result(conn);
                // lead time: 1.53ms

                let stream = res_stream?;
                let stream_id = stream.id;
                let user_id = stream.user_id;

                // Update the list of "tags" for the specified "stream".
                let res_update_stream_tags = self.update_list_stream_tags(conn, stream_id, user_id, tags);
                // lead time: 1.73ms

                if let Err(err) = res_update_stream_tags {
                    err_table = "update_list_stream_tags";
                    return Err(err);
                };
                // Get a list of "tags" for the specified "stream".
                let res_stream_tags = self.get_stream_tags(conn, &[stream_id]);
                // lead time: 510.37µs

                let stream_tags = match res_stream_tags {
                    Ok(v) => v,
                    Err(err) => {
                        err_table = "get_stream_tags_names";
                        return Err(err);
                    }
                };

                Ok((stream, stream_tags))
            });
            // lead time: 4.48ms
            match res_data {
                Ok((stream, stream_tags)) => Ok((stream, stream_tags)),
                Err(err) => Err(format!("{}: {}", err_table, err.to_string())),
            }
        }
        /// Get the logo file name for an entity (stream) by ID.
        fn get_stream_logo_by_id(&self, id: i32) -> Result<Option<String>, String> {
            // Get a connection from the P2D2 pool.
            let mut conn = self.get_conn()?;

            // Run query using Diesel to find user by id and return it.
            let opt_stream = schema::streams::table
                .filter(streams_dsl::id.eq(id))
                .first::<Stream>(&mut conn)
                .optional()
                .map_err(|e| format!("find_stream_by_id: {}", e.to_string()))?;

            if let Some(stream) = opt_stream {
                Ok(stream.logo)
            } else {
                Ok(None)
            }
        }
        /// Modify an entity (stream).
        fn modify_stream(
            &self,
            id: i32,
            opt_user_id: Option<i32>,
            modify_stream: ModifyStream,
            opt_tags: Option<Vec<String>>,
        ) -> Result<Option<(Stream, Vec<StreamTagStreamId>)>, String> {
            // Get a connection from the P2D2 pool.
            let mut conn = self.get_conn()?;

            let mut err_table = "modify_stream";
            let res_data = conn.transaction::<_, diesel::result::Error, _>(|conn| {
                let res_stream = if modify_stream.is_empty() {
                    // Prepare an SQL query to get the stream.
                    let mut query = schema::streams::table.into_boxed();
                    // Add a filter by unique stream identifier.
                    query = query.filter(streams_dsl::id.eq(id));
                    if let Some(user_id) = opt_user_id {
                        // Add an additional filter by user ID.
                        query = query.filter(streams_dsl::user_id.eq(user_id));
                    }
                    // Run query using Diesel to get the entry (stream).
                    query.first::<Stream>(conn).optional()
                } else {
                    // Prepare a SQL-request to update the entry (stream).
                    let mut query = diesel::update(schema::streams::table).into_boxed();
                    // Add a filter by unique stream identifier.
                    query = query.filter(streams_dsl::id.eq(id));
                    if let Some(user_id) = opt_user_id {
                        // Add an additional filter by user ID.
                        query = query.filter(streams_dsl::user_id.eq(user_id));
                    }
                    // Run query using Diesel to update the entry (stream).
                    query
                        .set(&modify_stream)
                        .returning(Stream::as_returning())
                        .get_result(conn)
                        .optional()
                };

                let opt_stream = res_stream?;
                // let opt_stream = res_stream_info.map_err(|e| format!("delete_stream: {}", e.to_string()))?;

                if let Some(stream) = opt_stream {
                    let stream_id = stream.id;
                    let user_id = stream.user_id;

                    if let Some(tags) = opt_tags {
                        // Update the list of "tags" for the specified "stream".
                        let res_update_stream_tags = self.update_list_stream_tags(conn, stream_id, user_id, &tags);
                        // lead time: 1.04ms

                        if let Err(err) = res_update_stream_tags {
                            err_table = "update_list_stream_tags";
                            return Err(err);
                        };
                    }

                    // Get a list of "tags" for the specified "stream".
                    let res_stream_tags = self.get_stream_tags(conn, &[stream_id]);
                    // lead time: 532.19µs

                    let stream_tags = match res_stream_tags {
                        Ok(v) => v,
                        Err(err) => {
                            err_table = "get_stream_tags_names";
                            return Err(err);
                        }
                    };
                    Ok(Some((stream, stream_tags)))
                } else {
                    Ok(None)
                }
            });
            // lead time: 3.84ms
            match res_data {
                Ok(value) => Ok(value),
                Err(err) => Err(format!("{}: {}", err_table, err.to_string())),
            }
        }
        /// Delete an entity (stream).
        fn delete_stream(
            &self,
            id: i32,
            opt_user_id: Option<i32>,
        ) -> Result<Option<(Stream, Vec<StreamTagStreamId>)>, String> {
            // Get a connection from the P2D2 pool.
            let mut conn = self.get_conn()?;

            // Get a list of "tags" for the specified "stream".
            let stream_tags = self.get_stream_tags(&mut conn, &[id]).map_err(|e| e.to_string())?;

            // Prepare a SQL-request to delete the entry (stream).
            let mut query = diesel::delete(schema::streams::table).into_boxed();
            // Add a filter by unique stream identifier.
            query = query.filter(streams_dsl::id.eq(id));
            if let Some(user_id) = opt_user_id {
                // Add an additional filter by user ID.
                query = query.filter(streams_dsl::user_id.eq(user_id));
            }
            // Run query using Diesel to delete the entry (stream).
            let res_stream_info = query.returning(Stream::as_returning()).get_result(&mut conn).optional();

            let opt_stream = res_stream_info.map_err(|e| format!("delete_stream: {}", e.to_string()))?;
            if let Some(stream) = opt_stream {
                // Update the "stream_tags" data for user.
                let _ = self.update_stream_tags_for_user(&mut conn, stream.user_id);

                Ok(Some((stream, stream_tags)))
            } else {
                Ok(None)
            }
        }
    }
}

#[cfg(feature = "mockdata")]
pub mod tests {
    use std::cmp::Ordering;

    use chrono::Duration;

    use crate::streams::stream_models::{self, StreamInfoDto};

    use super::*;

    pub const STREAM_ID: i32 = 1400;

    #[derive(Debug, Clone)]
    pub struct StreamOrmApp {
        pub stream_info_vec: Vec<StreamInfoDto>,
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
        pub fn create(stream_vec: &[StreamInfoDto]) -> Self {
            let mut stream_info_vec: Vec<StreamInfoDto> = Vec::new();

            for (idx, stream) in stream_vec.iter().enumerate() {
                let mut stream2 = stream.clone();
                let delta: i32 = idx.try_into().unwrap();
                stream2.id = STREAM_ID + delta;
                stream_info_vec.push(stream2);
            }
            StreamOrmApp { stream_info_vec }
        }
        /// Create entity "Stream" from "StreamInfoDto".
        fn to_stream(stream_info: &StreamInfoDto) -> Stream {
            Stream {
                id: stream_info.id,
                user_id: stream_info.user_id,
                title: stream_info.title.to_owned(),
                descript: stream_info.descript.to_owned(),
                logo: stream_info.logo.clone(),
                starttime: stream_info.starttime.clone(),
                live: stream_info.live,
                state: stream_info.state.clone(),
                started: stream_info.started.clone(),
                stopped: stream_info.stopped.clone(),
                source: stream_info.source.to_owned(),
                created_at: stream_info.created_at.clone(),
                updated_at: stream_info.updated_at.clone(),
            }
        }
        /// Get a list of "tags" for the specified "stream".
        fn get_tags(&self, stream_info: &StreamInfoDto) -> Vec<StreamTagStreamId> {
            let mut result: Vec<StreamTagStreamId> = Vec::new();
            let mut id = 0;
            for tag in stream_info.tags.iter() {
                let value = StreamTagStreamId {
                    stream_id: stream_info.id,
                    id,
                    user_id: stream_info.user_id,
                    name: tag.to_owned(),
                };
                result.push(value);
                id += 1;
            }
            result
        }
        /// Create entity "StreamTagStreamId" from "Stream".
        fn create_stream_tags(stream_id: i32, user_id: i32, tags: &[String]) -> Vec<StreamTagStreamId> {
            let mut stream_tags: Vec<StreamTagStreamId> = vec![];
            let mut tag_id = 0;
            for tag in tags.iter() {
                stream_tags.push(StreamTagStreamId {
                    stream_id,
                    id: tag_id,
                    user_id,
                    name: tag.to_string(),
                });
                tag_id += 1;
            }
            stream_tags
        }
    }

    impl StreamOrm for StreamOrmApp {
        /// Find for an entity (stream) by id.
        #[rustfmt::skip]
        fn find_stream_by_id(&self, id: i32, opt_user_id: Option<i32>) -> Result<Option<(Stream, Vec<StreamTagStreamId>)>, String> {
            let opt_stream_info = self
                .stream_info_vec
                .iter()
                .find(|stream| {
                    #[rustfmt::skip]
                    let val_user_id = match opt_user_id { Some(v) => v, None => stream.user_id };
                    stream.id == id && stream.user_id == val_user_id
                })
                .map(|stream| stream.clone());

            if let Some(stream_info) = opt_stream_info {
                let stream_tags = self.get_tags(&stream_info);
                Ok(Some((Self::to_stream(&stream_info), stream_tags)))
            } else {
                Ok(None)
            }
        }
        /// Find for an entity (stream) by SearchStreamInfoDto.
        fn find_streams(
            &self,
            search_stream: SearchStream,
        ) -> Result<(u32, Vec<Stream>, Vec<StreamTagStreamId>), String> {
            let mut streams_info: Vec<StreamInfoDto> = vec![];

            let is_future = search_stream.is_future.is_some();
            #[rustfmt::skip]
            let is_future_val = if is_future { search_stream.is_future.unwrap() } else { false };

            let now = Utc::now();
            let now_date = now.date_naive();

            for stream in self.stream_info_vec.iter() {
                let mut is_add_value = true;

                if stream.user_id != search_stream.user_id {
                    is_add_value = false;
                }
                if stream.live != search_stream.live.unwrap_or(stream.live) {
                    is_add_value = false;
                }
                let starttime_date = stream.starttime.date_naive();

                if is_future && !is_future_val && starttime_date >= now_date {
                    is_add_value = false;
                }
                if is_future && is_future_val && starttime_date < now_date {
                    is_add_value = false;
                }

                if is_add_value {
                    streams_info.push(stream.clone());
                }
            }

            let order_column = search_stream.order_column.unwrap_or(stream_models::SEARCH_STREAM_ORDER_COLUMN);
            let is_order_starttime = order_column == stream_models::OrderColumn::Starttime;
            let order_direction = search_stream
                .order_direction
                .unwrap_or(stream_models::SEARCH_STREAM_ORDER_DIRECTION);
            let is_order_asc = order_direction == stream_models::OrderDirection::Asc;

            streams_info.sort_by(|a, b| {
                let mut result = if is_order_starttime {
                    a.starttime.partial_cmp(&b.starttime).unwrap_or(Ordering::Equal)
                } else {
                    a.title.to_lowercase().cmp(&b.title.to_lowercase())
                };
                if !is_order_asc {
                    result = match result {
                        Ordering::Less => Ordering::Greater,
                        Ordering::Greater => Ordering::Less,
                        Ordering::Equal => Ordering::Equal,
                    };
                }
                result
            });

            let amount = streams_info.len();
            let page = search_stream.page.unwrap_or(stream_models::SEARCH_STREAM_PAGE);
            let limit = search_stream.limit.unwrap_or(stream_models::SEARCH_STREAM_LIMIT);
            let min_idx = (page - 1) * limit;
            let max_idx = min_idx + limit;
            let mut idx = 0;
            streams_info.retain(|_| {
                let res = min_idx <= idx && idx < max_idx;
                idx += 1;
                res
            });

            let count: u32 = amount.try_into().unwrap();

            let mut streams: Vec<Stream> = vec![];
            let mut stream_tags: Vec<StreamTagStreamId> = vec![];
            let mut tag_id = 0;
            for stream in streams_info.iter() {
                streams.push(Self::to_stream(stream));
                for tag in stream.tags.iter() {
                    #[rustfmt::skip]
                    stream_tags.push(StreamTagStreamId {
                        stream_id: stream.id, id: tag_id, user_id: stream.user_id, name: tag.to_string()
                    });
                    tag_id += 1;
                }
            }

            Ok((count, streams, stream_tags))
        }
        /// Find for an entity (stream event) by SearchStreamEvent.
        fn find_stream_events(&self, search_stream_event: SearchStreamEvent) -> Result<(u32, Vec<Stream>), String> {
            let mut streams_info: Vec<StreamInfoDto> = vec![];

            let start = search_stream_event.starttime;
            let finish = start + Duration::hours(24);

            for stream in self.stream_info_vec.iter() {
                if stream.user_id == search_stream_event.user_id
                    && start <= stream.starttime
                    && stream.starttime < finish
                {
                    streams_info.push(stream.clone());
                }
            }

            streams_info.sort_by(|a, b| a.starttime.partial_cmp(&b.starttime).unwrap_or(Ordering::Equal));

            let amount = streams_info.len();
            let page = search_stream_event.page.unwrap_or(stream_models::SEARCH_STREAM_EVENT_PAGE);
            let limit = search_stream_event.limit.unwrap_or(stream_models::SEARCH_STREAM_EVENT_LIMIT);
            let min_idx = (page - 1) * limit;
            let max_idx = min_idx + limit;
            let mut idx = 0;
            streams_info.retain(|_| {
                let res = min_idx <= idx && idx < max_idx;
                idx += 1;
                res
            });

            let count: u32 = amount.try_into().unwrap();

            let mut streams: Vec<Stream> = vec![];
            let mut stream_tags: Vec<StreamTagStreamId> = vec![];
            let mut tag_id = 0;
            for stream in streams_info.iter() {
                streams.push(Self::to_stream(stream));
                for tag in stream.tags.iter() {
                    #[rustfmt::skip]
                    stream_tags.push(StreamTagStreamId {
                        stream_id: stream.id, id: tag_id, user_id: stream.user_id, name: tag.to_string()
                    });
                    tag_id += 1;
                }
            }

            Ok((count, streams))
        }
        /// Find for an entity (stream period) by SearchStreamPeriod.
        fn find_streams_period(&self, search_stream_period: SearchStreamPeriod) -> Result<Vec<DateTime<Utc>>, String> {
            let mut streams_info: Vec<StreamInfoDto> = vec![];

            let start = search_stream_period.start;
            let finish = search_stream_period.finish;

            for stream in self.stream_info_vec.iter() {
                if stream.user_id == search_stream_period.user_id
                    && start <= stream.starttime
                    && stream.starttime <= finish
                {
                    streams_info.push(stream.clone());
                }
            }

            streams_info.sort_by(|a, b| a.starttime.partial_cmp(&b.starttime).unwrap_or(Ordering::Equal));

            let list: Vec<DateTime<Utc>> = streams_info.into_iter().map(|v| v.starttime).collect();

            Ok(list)
        }
        /// Add a new entity (stream).
        fn create_stream(
            &self,
            create_stream: CreateStream,
            tags: &[String],
        ) -> Result<(Stream, Vec<StreamTagStreamId>), String> {
            let len: i32 = self.stream_info_vec.len().try_into().unwrap(); // convert usize as i32
            let stream_saved = Stream::create(create_stream, STREAM_ID + len);
            let stream_tags = Self::create_stream_tags(stream_saved.id, stream_saved.user_id, tags);

            Ok((stream_saved, stream_tags))
        }
        /// Get the logo file name for an entity (stream) by ID.
        fn get_stream_logo_by_id(&self, id: i32) -> Result<Option<String>, String> {
            let stream_info_opt = self
                .stream_info_vec
                .iter()
                .find(|stream| stream.id == id)
                .map(|stream| stream.clone());

            if let Some(stream_info) = stream_info_opt {
                Ok(stream_info.logo)
            } else {
                Ok(None)
            }
        }
        /// Modify an entity (stream).
        fn modify_stream(
            &self,
            id: i32,
            opt_user_id: Option<i32>,
            modify_stream: ModifyStream,
            opt_tags: Option<Vec<String>>,
        ) -> Result<Option<(Stream, Vec<StreamTagStreamId>)>, String> {
            let opt_stream_info = if let Some(user_id) = opt_user_id {
                self.stream_info_vec
                    .iter()
                    .find(|stream| stream.id == id && stream.user_id == user_id)
                    .map(|stream| stream.clone())
            } else {
                self.stream_info_vec
                    .iter()
                    .find(|stream| stream.id == id)
                    .map(|stream| stream.clone())
            };

            if let Some(stream_info) = opt_stream_info {
                #[rustfmt::skip]
                let logo = match modify_stream.logo { Some(logo) => logo, None => stream_info.logo };
                let stream_saved = Stream {
                    id: stream_info.id,
                    user_id: stream_info.user_id,
                    title: modify_stream.title.unwrap_or(stream_info.title.to_string()),
                    descript: modify_stream.descript.unwrap_or(stream_info.descript.to_string()),
                    logo,
                    starttime: modify_stream.starttime.unwrap_or(stream_info.starttime.clone()),
                    live: stream_info.live,
                    state: stream_info.state.clone(),
                    started: stream_info.started.clone(),
                    stopped: stream_info.stopped.clone(),
                    source: modify_stream.source.unwrap_or(stream_info.source.to_string()),
                    created_at: stream_info.created_at,
                    updated_at: Utc::now(),
                };
                let new_tags: Vec<String> = match opt_tags {
                    Some(value) => value,
                    None => stream_info.tags.clone(),
                };
                let stream_tags = Self::create_stream_tags(stream_info.id, stream_info.user_id, &new_tags);

                Ok(Some((stream_saved, stream_tags)))
            } else {
                Ok(None)
            }
        }
        /// Delete an entity (stream).
        fn delete_stream(
            &self,
            id: i32,
            opt_user_id: Option<i32>,
        ) -> Result<Option<(Stream, Vec<StreamTagStreamId>)>, String> {
            let opt_stream_info = if let Some(user_id) = opt_user_id {
                self.stream_info_vec
                    .iter()
                    .find(|stream| stream.id == id && stream.user_id == user_id)
            } else {
                self.stream_info_vec.iter().find(|stream| stream.id == id)
            };

            match opt_stream_info {
                Some(stream_info) => Ok(Some((Self::to_stream(stream_info), self.get_tags(stream_info)))),
                None => Ok(None),
            }
        }
    }
}
