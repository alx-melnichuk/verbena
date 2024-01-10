use super::stream_models::{CreateStream, ModifyStream, SearchStream, Stream, StreamTagStreamId};

pub trait StreamOrm {
    /// Find for an entity (stream) by id.
    fn find_stream_by_id(&self, id: i32) -> Result<Option<Stream>, String>;
    /// Find for an entity (stream) by SearchStreamInfoDto.
    fn find_streams(&self, search_stream: SearchStream) -> Result<(Vec<Stream>, u32), String>;
    /// Get a list of "tags" for the specified "stream".
    fn get_stream_tags_by_id(&self, ids: &[i32]) -> Result<Vec<StreamTagStreamId>, String>;
    /// Find for an entity (stream) by SearchStreamInfoDto.
    fn find_streams2(&self, search_stream: SearchStream) -> Result<(u32, Vec<Stream>, Vec<StreamTagStreamId>), String>;
    /// Add a new entity (stream).
    fn create_stream(&self, create_stream: CreateStream, tags: &[String]) -> Result<Stream, String>;
    /// Modify an entity (stream).
    fn modify_stream(&self, id: i32, modify_stream: ModifyStream) -> Result<Option<Stream>, String>;
    /// Update a list of "stream_tags" for the entity (stream).
    fn update_stream_tags(&self, id: i32, user_id: i32, tags: Vec<String>) -> Result<(), String>;
    /// Delete an entity (stream).
    fn delete_stream(&self, id: i32) -> Result<usize, String>;
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
    use chrono::Utc;
    use diesel::{self, prelude::*, sql_types};
    use diesel::{debug_query, pg::Pg};
    use schema::streams::dsl;

    use crate::dbase;
    use crate::schema;
    use crate::streams::stream_models::{self, CreateStream};

    use super::*;

    pub const CONN_POOL: &str = "ConnectionPool";
    // pub const MAX_LIMIT_STREAM_TAGS: i64 = stream_models::TAG_NAME_MAX_AMOUNT as i64;

    #[derive(Debug, Clone)]
    pub struct StreamOrmApp {
        pub pool: dbase::DbPool,
    }

    impl StreamOrmApp {
        pub fn new(pool: dbase::DbPool) -> Self {
            StreamOrmApp { pool }
        }
        pub fn get_conn(&self) -> Result<dbase::DbPooledConnection, String> {
            (&self.pool).get().map_err(|e| format!("{CONN_POOL}: {}", e.to_string()))
        }
    }

    impl StreamOrm for StreamOrmApp {
        /// Find for an entity (stream) by id.
        fn find_stream_by_id(&self, id: i32) -> Result<Option<Stream>, String> {
            // Get a connection from the P2D2 pool.
            let mut conn = self.get_conn()?;

            // Run query using Diesel to find user by id and return it.
            let opt_stream = schema::streams::table
                .filter(dsl::id.eq(id))
                .first::<Stream>(&mut conn)
                .optional()
                .map_err(|e| format!("find_stream_by_id: {}", e.to_string()))?;

            Ok(opt_stream)
        }
        /// Find for an entity (stream) by SearchStream.
        fn find_streams(&self, search_stream: SearchStream) -> Result<(Vec<Stream>, u32), String> {
            // Get a connection from the P2D2 pool.
            let mut conn = self.get_conn()?;
            dbg!(&search_stream);
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
                .filter(dsl::status.eq(true))
                .offset(offset.into())
                .limit(limit.into());

            // Create a query to get the number of elements in the list of "threads".
            let mut query_count = schema::streams::table.into_boxed();
            query_count = query_count.filter(dsl::status.eq(true));

            if let Some(user_id) = search_stream.user_id {
                query_list = query_list.filter(dsl::user_id.eq(user_id));
                query_count = query_count.filter(dsl::user_id.eq(user_id));
            }
            if let Some(live) = search_stream.live {
                query_list = query_list.filter(dsl::live.eq(live));
                query_count = query_count.filter(dsl::live.eq(live));
            }
            if let Some(is_future) = search_stream.is_future {
                let now_date = Utc::now().date_naive().and_hms_opt(0, 0, 0).unwrap();
                if !is_future {
                    // starttime < now_date
                    query_list = query_list.filter(dsl::starttime.lt(now_date));
                    query_count = query_count.filter(dsl::starttime.lt(now_date));
                } else {
                    // starttime >= now_date
                    query_list = query_list.filter(dsl::starttime.ge(now_date));
                    query_count = query_count.filter(dsl::starttime.ge(now_date));
                }
            }

            if order_column == stream_models::OrderColumn::Starttime {
                if is_asc {
                    query_list = query_list.order_by(dsl::starttime.asc());
                } else {
                    query_list = query_list.order_by(dsl::starttime.desc());
                }
            } else {
                if is_asc {
                    query_list = query_list.order_by(dsl::title.asc());
                } else {
                    query_list = query_list.order_by(dsl::title.desc());
                }
            }

            // #eprintln!("\nquery_count_sql: {}\n", debug_query::<Pg, _>(&query_count).to_string());
            // #let query_list_sql = debug_query::<Pg, _>(&query_list).to_string();
            // #eprintln!("\nquery_list_sql: {}\n", query_list_sql);

            let amount_res = query_count.count().get_result::<i64>(&mut conn);
            if let Err(err) = amount_res {
                return Err(format!("find_streams_by_user_id: (query_count) {}", err));
            }
            let amount: i64 = amount_res.unwrap();
            let count: u32 = amount.try_into().unwrap();

            let stream_vec: Vec<Stream> = query_list
                .load(&mut conn)
                .map_err(|e| format!("find_streams_by_user_id: (query_list) {}", e.to_string()))?;

            Ok((stream_vec, count))
        }
        /// Get a list of "tags" for the specified "stream".
        fn get_stream_tags_by_id(&self, ids: &[i32]) -> Result<Vec<StreamTagStreamId>, String> {
            // Get a connection from the P2D2 pool.
            let mut conn = self.get_conn()?;

            let query = diesel::sql_query(format!(
                "{}{}{}{}{}",
                "SELECT L.stream_id, T.* ",
                " FROM link_stream_tags_to_streams L, stream_tags T, ",
                " (SELECT a AS id FROM unnest($1) AS a) B ",
                " WHERE L.stream_tag_id = T.id and L.stream_id = B.id ",
                " ORDER BY L.stream_id ASC, T.id ASC "
            ))
            .bind::<sql_types::Array<sql_types::Integer>, _>(ids);

            let result = query
                .get_results::<StreamTagStreamId>(&mut conn)
                .map_err(|e| format!("get_stream_tags_by_id: {}", e.to_string()))?;

            Ok(result)
        }
        /// Find for an entity (stream) by SearchStream.
        fn find_streams2(
            &self,
            search_stream: SearchStream,
        ) -> Result<(u32, Vec<Stream>, Vec<StreamTagStreamId>), String> {
            // Get a connection from the P2D2 pool.
            let mut conn = self.get_conn()?;
            dbg!(&search_stream);
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
                .filter(dsl::status.eq(true))
                .offset(offset.into())
                .limit(limit.into());

            // Create a query to get the number of elements in the list of "threads".
            let mut query_count = schema::streams::table.into_boxed();
            query_count = query_count.filter(dsl::status.eq(true));

            if let Some(user_id) = search_stream.user_id {
                query_list = query_list.filter(dsl::user_id.eq(user_id));
                query_count = query_count.filter(dsl::user_id.eq(user_id));
            }
            if let Some(live) = search_stream.live {
                query_list = query_list.filter(dsl::live.eq(live));
                query_count = query_count.filter(dsl::live.eq(live));
            }
            if let Some(is_future) = search_stream.is_future {
                let now_date = Utc::now().date_naive().and_hms_opt(0, 0, 0).unwrap();
                if !is_future {
                    // starttime < now_date
                    query_list = query_list.filter(dsl::starttime.lt(now_date));
                    query_count = query_count.filter(dsl::starttime.lt(now_date));
                } else {
                    // starttime >= now_date
                    query_list = query_list.filter(dsl::starttime.ge(now_date));
                    query_count = query_count.filter(dsl::starttime.ge(now_date));
                }
            }

            if order_column == stream_models::OrderColumn::Starttime {
                if is_asc {
                    query_list = query_list.order_by(dsl::starttime.asc());
                } else {
                    query_list = query_list.order_by(dsl::starttime.desc());
                }
            } else {
                if is_asc {
                    query_list = query_list.order_by(dsl::title.asc());
                } else {
                    query_list = query_list.order_by(dsl::title.desc());
                }
            }

            // #eprintln!("\nquery_count_sql: {}\n", debug_query::<Pg, _>(&query_count).to_string());
            // #let query_list_sql = debug_query::<Pg, _>(&query_list).to_string();
            // #eprintln!("\nquery_list_sql: {}\n", query_list_sql);

            let amount_res = query_count.count().get_result::<i64>(&mut conn);
            if let Err(err) = amount_res {
                return Err(format!("find_streams_by_user_id: (query_count) {}", err));
            }
            let amount: i64 = amount_res.unwrap();
            let count: u32 = amount.try_into().unwrap();

            let streams: Vec<Stream> = query_list
                .load(&mut conn)
                .map_err(|e| format!("find_streams_by_user_id: (query_list) {}", e.to_string()))?;

            // let (streams, count) = match res_streams {
            //     Ok(v) => v,
            //     Err(e) => return Err(e),
            // };
            // Get a list of "stream" identifiers.
            let ids: Vec<i32> = streams.iter().map(|stream| stream.id).collect();
            // Get a list of "tags" for the specified "stream".
            // let ids: &[i32] = vec![];
            let query = diesel::sql_query(format!(
                "{}{}{}{}{}",
                "SELECT L.stream_id, T.* ",
                " FROM link_stream_tags_to_streams L, stream_tags T, ",
                " (SELECT a AS id FROM unnest($1) AS a) B ",
                " WHERE L.stream_tag_id = T.id and L.stream_id = B.id ",
                " ORDER BY L.stream_id ASC, T.id ASC "
            ))
            .bind::<sql_types::Array<sql_types::Integer>, _>(ids);

            let stream_tags = query
                .get_results::<StreamTagStreamId>(&mut conn)
                .map_err(|e| format!("get_stream_tags_by_id: {}", e.to_string()))?;

            Ok((count, streams, stream_tags))
        }

        /// Add a new entity (stream).
        fn create_stream(&self, create_stream: CreateStream, tags: &[String]) -> Result<Stream, String> {
            // Get a connection from the P2D2 pool.
            let mut conn = self.get_conn()?;
            let mut is_err_tags = false;

            let res_stream = conn.transaction::<_, diesel::result::Error, _>(|conn| {
                // Run query using Diesel to add a new entry (stream).
                let res_stream = diesel::insert_into(schema::streams::table)
                    .values(create_stream)
                    .returning(Stream::as_returning())
                    .get_result(conn);

                if let Ok(ref stream) = res_stream {
                    let tag_names = tags.join(stream_models::TAG_NAME_SEPARATOR);
                    #[rustfmt::skip]
                    let user_id = if 222 == stream.user_id { stream.user_id + 200 } else { stream.user_id };
                    // Run query using Diesel to add a list of "stream_tags" for the entity (stream).
                    let query = diesel::sql_query("CALL update_list_stream_tag_to_stream($1, $2, $3);")
                        .bind::<sql_types::Integer, _>(stream.id)
                        .bind::<sql_types::Integer, _>(user_id)
                        .bind::<sql_types::VarChar, _>(tag_names);
                    let query_sql = debug_query::<Pg, _>(&query).to_string();
                    eprintln!("query_sql: {}", query_sql);

                    let res_stream_tag = query.execute(conn);

                    if let Err(err) = res_stream_tag {
                        is_err_tags = true;
                        return Err(err);
                    }
                }
                res_stream
            });

            match res_stream {
                Ok(stream) => Ok(stream),
                Err(err) => {
                    #[rustfmt::skip]
                    let msg = if !is_err_tags { "create_stream" } else { "update_stream_tags" };
                    Err(format!("{}: {}", msg, err.to_string()))
                }
            }
            /*
            // Run query using Diesel to add a new entry (stream).
            let stream = diesel::insert_into(schema::streams::table)
                .values(create_stream)
                .returning(Stream::as_returning())
                .get_result(&mut conn)
                .map_err(|e| format!("create_stream: {}", e.to_string()))?;

            Ok(stream)
            */
        }
        /// Modify an entity (stream).
        fn modify_stream(&self, id: i32, modify_stream: ModifyStream) -> Result<Option<Stream>, String> {
            // Get a connection from the P2D2 pool.
            let mut conn = self.get_conn()?;

            // Run query using Diesel to full or partially modify the user entry.
            let result = diesel::update(dsl::streams.find(id))
                .set(&modify_stream)
                .returning(Stream::as_returning())
                .get_result(&mut conn)
                .optional()
                .map_err(|e| format!("modify_stream: {}", e.to_string()))?;

            Ok(result)
        }
        /// Update a list of "stream_tags" for the entity (stream).
        fn update_stream_tags(&self, id: i32, user_id: i32, tags: Vec<String>) -> Result<(), String> {
            // Get a connection from the P2D2 pool.
            let mut conn = self.get_conn()?;

            let tag_names = tags.join(stream_models::TAG_NAME_SEPARATOR);
            // Run query using Diesel to add a list of "stream_tags" for the entity (stream).
            let query = diesel::sql_query("CALL update_list_stream_tag_to_stream($1, $2, $3);")
                .bind::<sql_types::Integer, _>(id)
                .bind::<sql_types::Integer, _>(user_id)
                .bind::<sql_types::VarChar, _>(tag_names);
            let query_sql = debug_query::<Pg, _>(&query).to_string();
            eprintln!("query_sql: {}", query_sql);

            query
                .execute(&mut conn)
                .map_err(|e| format!("update_stream_tags: {}", e.to_string()))?;

            Ok(())
        }
        /// Delete an entity (stream).
        fn delete_stream(&self, id: i32) -> Result<usize, String> {
            // Get a connection from the P2D2 pool.
            let mut conn = self.get_conn()?;

            // Run query using Diesel to delete a entry (stream).
            let count: usize = diesel::delete(dsl::streams.find(id))
                .execute(&mut conn)
                .map_err(|e| format!("delete_stream: {}", e.to_string()))?;

            Ok(count)
        }
    }
}

#[cfg(feature = "mockdata")]
pub mod tests {
    use std::cmp::Ordering;

    use chrono::Utc;

    use crate::streams::stream_models;

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
        fn to_stream(stream_info: StreamInfoDto) -> Stream {
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
                status: stream_info.status,
                source: stream_info.source.to_owned(),
                created_at: stream_info.created_at.clone(),
                updated_at: stream_info.updated_at.clone(),
            }
        }
    }

    impl StreamOrm for StreamOrmApp {
        /// Find for an entity (stream) by id.
        fn find_stream_by_id(&self, id: i32) -> Result<Option<Stream>, String> {
            let stream_info_opt = self
                .stream_info_vec
                .iter()
                .find(|stream| stream.id == id)
                .map(|stream| stream.clone());

            if let Some(mut stream_info) = stream_info_opt {
                Ok(Some(Self::to_stream(stream_info)))
            } else {
                Ok(None)
            }
        }
        /// Find for an entity (stream) by SearchStream.
        fn find_streams(&self, search_stream: SearchStream) -> Result<(Vec<Stream>, u32), String> {
            let mut stream_vec: Vec<Stream> = vec![];

            let is_user_id = search_stream.user_id.is_some();
            let is_live = search_stream.live.is_some();
            let is_future = search_stream.is_future.is_some();
            #[rustfmt::skip]
            let is_future_value = if is_future { search_stream.is_future.unwrap() } else { false };

            let is_check = is_user_id || is_live || is_future;
            let now = Utc::now();
            let now_date = now.date_naive();

            for stream in self.stream_info_vec.iter() {
                if !stream.status {
                    continue;
                }
                let mut is_add_value = !is_check;

                if !is_add_value && is_user_id && stream.user_id == search_stream.user_id.unwrap() {
                    is_add_value = true;
                }
                if !is_add_value && is_live && stream.live == search_stream.live.unwrap() {
                    is_add_value = true;
                }
                let starttime_date = stream.starttime.date_naive();
                if !is_add_value
                    && is_future
                    && ((!is_future_value && starttime_date < now_date)
                        || (is_future_value && starttime_date >= now_date))
                {
                    is_add_value = true;
                }

                if is_add_value {
                    stream_vec.push(Self::to_stream(stream.clone()));
                }
            }

            let order_column = search_stream.order_column.unwrap_or(stream_models::SEARCH_STREAM_ORDER_COLUMN);
            let is_order_starttime = order_column == stream_models::OrderColumn::Starttime;
            let order_direction = search_stream
                .order_direction
                .unwrap_or(stream_models::SEARCH_STREAM_ORDER_DIRECTION);
            let is_order_asc = order_direction == stream_models::OrderDirection::Asc;

            stream_vec.sort_by(|a, b| {
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

            let amount = stream_vec.len();
            let page = search_stream.page.unwrap_or(stream_models::SEARCH_STREAM_PAGE);
            let limit = search_stream.limit.unwrap_or(stream_models::SEARCH_STREAM_LIMIT);
            let min_idx = (page - 1) * limit;
            let max_idx = min_idx + limit;
            let mut idx = 0;
            stream_vec.retain(|_| {
                let res = min_idx <= idx && idx < max_idx;
                idx += 1;
                res
            });

            let count: u32 = amount.try_into().unwrap();

            Ok((stream_vec, count))
        }
        /// Get a list of "tags" for the specified "stream".
        fn get_stream_tags_by_id(&self, ids: &[i32]) -> Result<Vec<StreamTagStreamId>, String> {
            let mut result: Vec<StreamTagStreamId> = Vec::new();

            let iter_streams_info = self
                .stream_info_vec
                .iter()
                .filter(|stream| ids.contains(&stream.id))
                .map(|stream| stream.clone());

            let mut id = 0;
            for stream_info in iter_streams_info {
                for tag in stream_info.tags {
                    let value = StreamTagStreamId {
                        stream_id: stream_info.id,
                        id,
                        user_id: stream_info.user_id,
                        name: tag.to_owned(),
                    };
                    result.push(value);
                    id += 1;
                }
            }
            Ok(result)
        }
        /// Find for an entity (stream) by SearchStreamInfoDto.
        fn find_streams2(
            &self,
            search_stream: SearchStream,
        ) -> Result<(u32, Vec<Stream>, Vec<StreamTagStreamId>), String> {
            Ok((0, vec![], vec![]))
        }
        /// Add a new entity (stream).
        fn create_stream(&self, create_stream: CreateStream, tags: &[String]) -> Result<Stream, String> {
            let now = Utc::now();
            let len: i32 = self.stream_info_vec.len().try_into().unwrap(); // convert usize as i32
            let stream_descript_def = stream_models::STREAM_DESCRIPT_DEF;
            let stream_source_def = stream_models::STREAM_SOURCE_DEF;

            let stream_saved = Stream {
                id: STREAM_ID + len,
                user_id: create_stream.user_id,
                title: create_stream.title.to_owned(),
                descript: create_stream.descript.clone().unwrap_or(stream_descript_def.to_string()),
                logo: create_stream.logo.clone(),
                starttime: create_stream.starttime,
                live: create_stream.live.unwrap_or(stream_models::STREAM_LIVE_DEF),
                state: create_stream.state.unwrap_or(stream_models::STREAM_STATE_DEF),
                started: create_stream.started.clone(),
                stopped: create_stream.stopped.clone(),
                status: create_stream.status.unwrap_or(stream_models::STREAM_STATUS_DEF),
                source: create_stream.source.clone().unwrap_or(stream_source_def.to_string()),
                created_at: now,
                updated_at: now,
            };
            Ok(stream_saved)
        }

        /// Modify an entity (stream).
        fn modify_stream(&self, id: i32, modify_stream: ModifyStream) -> Result<Option<Stream>, String> {
            let user_id = modify_stream.user_id;

            let stream_opt = self
                .stream_info_vec
                .iter()
                .find(|stream| stream.id == id && stream.user_id == user_id)
                .map(|stream| stream.clone());

            if let Some(stream) = stream_opt {
                let stream_saved = Stream {
                    id: stream.id,
                    user_id: stream.user_id,
                    title: modify_stream.title.to_owned(),
                    descript: modify_stream.descript.to_owned(),
                    logo: modify_stream.logo.clone(),
                    starttime: modify_stream.starttime,
                    live: modify_stream.live,
                    state: modify_stream.state,
                    started: modify_stream.started.clone(),
                    stopped: modify_stream.stopped.clone(),
                    status: modify_stream.status,
                    source: modify_stream.source.to_owned(),
                    created_at: stream.created_at,
                    updated_at: Utc::now(),
                };
                Ok(Some(stream_saved))
            } else {
                Ok(None)
            }
        }
        /// Update a list of "stream_tags" for the entity (stream).
        fn update_stream_tags(&self, id: i32, user_id: i32, _: Vec<String>) -> Result<(), String> {
            let index_opt = self
                .stream_info_vec
                .iter()
                .position(|stream| stream.id == id && stream.user_id == user_id);

            if index_opt.is_some() {
                Ok(())
            } else {
                Err(format!("Not found id: {}, user_id: {}", id, user_id).to_string())
            }
        }
        /// Delete an entity (stream).
        fn delete_stream(&self, id: i32) -> Result<usize, String> {
            let stream_opt = self.stream_info_vec.iter().find(|stream| stream.id == id);

            #[rustfmt::skip]
            let result = if stream_opt.is_none() { 0 } else { 1 };
            Ok(result)
        }
    }
}
