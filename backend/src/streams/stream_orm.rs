use super::stream_models::{
    self, CreateStream, ModifyStream, SearchStreamInfoDto, Stream, StreamInfoDto,
};

pub trait StreamOrm {
    /// Find for an entity (stream) by id and user_id.
    fn find_stream_by_id(&self, id: i32, user_id: i32) -> Result<Option<StreamInfoDto>, String>;
    /// Find for an entity (stream) by SearchStreamInfoDto.
    fn find_streams(
        &self,
        search_stream: SearchStreamInfoDto,
        user_id: i32,
    ) -> Result<Vec<StreamInfoDto>, String>;
    /// Add a new entity (stream).
    fn create_stream(&self, create_stream: CreateStream) -> Result<Stream, String>;
    /// Modify an entity (stream).
    fn modify_stream(&self, id: i32, modify_stream: ModifyStream)
        -> Result<Option<Stream>, String>;
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
    use std::collections::HashMap;

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
        /// Get a list of "tags" for the specified "stream".
        fn get_stream_tags_by_id(
            &self,
            conn: &mut dbase::DbPooledConnection,
            ids: &[i32],
        ) -> Result<Vec<stream_models::StreamTagStreamId>, String> {
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
                .get_results::<stream_models::StreamTagStreamId>(conn)
                .map_err(|e| format!("get_stream_tags_by_id: {}", e.to_string()))?;

            Ok(result)
        }
        /// Merge a "stream" and a corresponding list of "tags".
        fn merge_streams_and_tags(
            &self,
            streams: &[Stream],
            stream_tags: &[stream_models::StreamTagStreamId],
            user_id: i32,
        ) -> Vec<StreamInfoDto> {
            let mut result: Vec<StreamInfoDto> = Vec::new();

            let mut tags_map: HashMap<i32, Vec<String>> = HashMap::new();
            #[rustfmt::skip]
            let mut curr_stream_id: i32 = if stream_tags.len() > 0 { stream_tags[0].stream_id } else { -1 };
            let mut tags: Vec<String> = vec![];
            for stream_tag in stream_tags.iter() {
                if curr_stream_id != stream_tag.stream_id {
                    tags_map.insert(curr_stream_id, tags.clone());
                    tags.clear();
                    curr_stream_id = stream_tag.stream_id;
                }
                tags.push(stream_tag.name.to_string());
            }
            tags_map.insert(curr_stream_id, tags.clone());

            for stream in streams.iter() {
                let stream = stream.clone();
                let mut tags: Vec<&str> = Vec::new();
                let tags_opt = tags_map.get(&stream.id);
                if let Some(tags_vec) = tags_opt {
                    let tags2: Vec<&str> = tags_vec.iter().map(|v| v.as_str()).collect();
                    tags.extend(tags2);
                }
                let stream_info_dto = StreamInfoDto::convert(stream, user_id, &tags);
                result.push(stream_info_dto);
            }
            result
        }
    }

    impl StreamOrm for StreamOrmApp {
        /// Find for an entity (stream) by id and user_id.
        fn find_stream_by_id(
            &self,
            id: i32,
            user_id: i32,
        ) -> Result<Option<StreamInfoDto>, String> {
            // Get a connection from the P2D2 pool.
            let mut conn = self.get_conn()?;

            // Run query using Diesel to find user by id and return it.
            let stream_opt = schema::streams::table
                .filter(dsl::id.eq(id))
                .first::<Stream>(&mut conn)
                .optional()
                .map_err(|e| format!("find_stream_by_id: {}", e.to_string()))?;

            if let Some(stream) = stream_opt {
                // Get a list of "tags" for the specified "stream".
                let stream_tags = self.get_stream_tags_by_id(&mut conn, &[id])?;
                // Merge a "stream" and a corresponding list of "tags".
                let result = self.merge_streams_and_tags(&[stream], &stream_tags, user_id);

                Ok(Some(result[0].clone()))
            } else {
                Ok(None)
            }
        }
        /// Find for an entity (stream) by SearchStreamInfoDto.
        fn find_streams(
            &self,
            search_stream: SearchStreamInfoDto,
            user_id: i32,
        ) -> Result<Vec<StreamInfoDto>, String> {
            // Get a connection from the P2D2 pool.
            let mut conn = self.get_conn()?;

            let page: i64 = search_stream.page.unwrap_or(1).into();
            let limit: i64 =
                search_stream.limit.unwrap_or(stream_models::SEARCH_STREAM_LIMIT).into();
            let offset: i64 = (page - 1) * limit;

            // let queryCount = this.streamRepository
            // .createQueryBuilder('streams')
            // .where('streams.status = :status', { status: true });

            // Build a query to find a list of "streams".
            let mut query_list = schema::streams::table.into_boxed();
            query_list =
                query_list.select(schema::streams::all_columns).filter(dsl::status.eq(true));

            if let Some(user_id) = search_stream.user_id {
                query_list = query_list.filter(dsl::user_id.eq(user_id));
            }
            // is_future
            if let Some(live) = search_stream.live {
                query_list = query_list.filter(dsl::live.eq(live));
            }

            query_list = query_list
                // .order_by(dsl::id.asc())
                // .then_order_by(dsl::user_id.asc())
                .offset(offset)
                .limit(limit);

            let query_list_sql = debug_query::<Pg, _>(&query_list).to_string();
            eprintln!("\nquery_list_sql: {}\n", query_list_sql);

            let stream_vec: Vec<Stream> = query_list
                .load(&mut conn)
                .map_err(|e| format!("find_streams_by_user_id: {}", e.to_string()))?;

            eprintln!("\nstream_vec.len(): {:?}\n", stream_vec.len());
            for stream in stream_vec.iter() {
                eprintln!("    stream: {:?}", stream);
            }
            // Get a list of "stream" identifiers.
            let ids: Vec<i32> = stream_vec.iter().map(|stream| stream.id).collect();
            // Get a list of "tags" for the specified "stream".
            let stream_tags = self.get_stream_tags_by_id(&mut conn, &ids)?;
            // Merge a "stream" and a corresponding list of "tags".
            let result = self.merge_streams_and_tags(&stream_vec, &stream_tags, user_id);

            // // Run query using Diesel to find user by user_id and return it (where final_date > now).
            // let result = schema::streams::table
            //     .filter(dsl::user_id.eq(user_id))
            //     .load::<Stream>(&mut conn)
            //     .map_err(|e| format!("find_streams_by_user_id: {}", e.to_string()))?;

            Ok(result)
        }

        /// Add a new entity (stream).
        fn create_stream(&self, create_stream: CreateStream) -> Result<Stream, String> {
            // Get a connection from the P2D2 pool.
            let mut conn = self.get_conn()?;

            // Run query using Diesel to add a new entry (stream).
            let stream = diesel::insert_into(schema::streams::table)
                .values(create_stream)
                .returning(Stream::as_returning())
                .get_result(&mut conn)
                .map_err(|e| format!("create_stream: {}", e.to_string()))?;

            Ok(stream)
        }

        /// Modify an entity (stream).
        fn modify_stream(
            &self,
            id: i32,
            modify_stream: ModifyStream,
        ) -> Result<Option<Stream>, String> {
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
        fn update_stream_tags(
            &self,
            id: i32,
            user_id: i32,
            tags: Vec<String>,
        ) -> Result<(), String> {
            // Get a connection from the P2D2 pool.
            let mut conn = self.get_conn()?;

            let tag_names = tags.join(",");
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
    // cell::RefCell, rc::Rc, collections::HashMap,
    use chrono::{DateTime, Utc};

    use crate::streams::stream_models::{CreateStream, ModifyStream, Stream, StreamState};

    use super::StreamOrm;

    pub const STREAM_ID: i32 = 1400;
    pub const DEF_DESCRIPT: &str = "";
    pub const DEF_LIVE: bool = false;
    pub const DEF_STATE: StreamState = StreamState::Waiting;
    pub const DEF_STATUS: bool = true;
    pub const DEF_SOURCE: &str = "obs";

    #[derive(Debug, Clone)]
    pub struct StreamOrmApp {
        pub stream_vec: Vec<Stream>,
        pub tags_vec: Vec<String>,
    }

    impl StreamOrmApp {
        /// Create a new instance.
        pub fn new() -> Self {
            StreamOrmApp {
                stream_vec: Vec::new(),
                tags_vec: Vec::new(),
            }
        }
        /// Create a new instance with the specified user list.
        #[cfg(test)]
        pub fn create(streams_tags: &[(Stream, &str)]) -> Self {
            let mut stream_vec: Vec<Stream> = Vec::new();
            let mut tags_vec: Vec<String> = Vec::new();
            // let mut idx: i32 = stream_list.len().try_into().unwrap();
            for (idx, (stream, tags)) in streams_tags.iter().enumerate() {
                let mut stream2 = stream.clone();
                let delta: i32 = idx.try_into().unwrap();
                stream2.id = STREAM_ID + delta;
                stream_vec.push(stream2);
                tags_vec.push(tags.to_string());
            }
            StreamOrmApp {
                stream_vec,
                tags_vec,
            }
        }
        // Create a new entity instance.
        #[cfg(test)]
        pub fn new_stream(id: i32, user_id: i32, title: &str, starttime: DateTime<Utc>) -> Stream {
            let now = Utc::now();
            Stream {
                id: id,
                user_id: user_id,
                title: title.to_owned(),
                descript: DEF_DESCRIPT.to_string(),
                logo: None,
                starttime: starttime.clone(),
                live: DEF_LIVE,
                state: DEF_STATE,
                started: None,
                stopped: None,
                status: DEF_STATUS,
                source: DEF_SOURCE.to_string(),
                created_at: now,
                updated_at: now,
            }
        }
    }

    impl StreamOrm for StreamOrmApp {
        /// Find for an entity (stream) by id and user_id.
        fn find_stream_by_id(
            &self,
            id: i32,
            user_id: i32,
        ) -> Result<Option<StreamInfoDto>, String> {
            eprintln!("find_stream_by_id(id: {})", id);
            let stream_opt = self
                .stream_vec
                .iter()
                .find(|stream| stream.id == id)
                .map(|stream| stream.clone());

            eprintln!("find_stream_by_id(): {:?}", &stream_opt);

            if let Some(stream) = stream_opt {
                // let res2 = self.get_stream_tags_by_id(&mut conn, id);

                let stream_info_dto = StreamInfoDto::convert(stream, user_id, &[]);
                Ok(Some(stream_info_dto))
            } else {
                Ok(None)
            }
        }
        /// Find for an entity (stream) by SearchStreamInfoDto.
        fn find_streams(
            &self,
            search_stream: SearchStreamInfoDto,
            user_id: i32,
        ) -> Result<Vec<StreamInfoDto>, String> {
            let result = self
                .stream_vec
                .iter()
                .filter(|stream| stream.user_id == user_id)
                .map(|stream| stream.clone())
                .collect();
            Ok(result)
        }
        /// Add a new entity (stream).
        fn create_stream(&self, create_stream: CreateStream) -> Result<Stream, String> {
            let now = Utc::now();
            let len: i32 = self.stream_vec.len().try_into().unwrap(); // convert usize as i32

            let stream_saved = Stream {
                id: STREAM_ID + len,
                user_id: create_stream.user_id,
                title: create_stream.title.to_owned(),
                descript: create_stream.descript.clone().unwrap_or(DEF_DESCRIPT.to_string()),
                logo: create_stream.logo.clone(),
                starttime: create_stream.starttime,
                live: create_stream.live.unwrap_or(DEF_LIVE),
                state: create_stream.state.unwrap_or(DEF_STATE),
                started: create_stream.started.clone(),
                stopped: create_stream.stopped.clone(),
                status: create_stream.status.unwrap_or(DEF_STATUS),
                source: create_stream.source.clone().unwrap_or(DEF_SOURCE.to_string()),
                created_at: now,
                updated_at: now,
            };
            Ok(stream_saved)
        }

        /// Modify an entity (stream).
        fn modify_stream(
            &self,
            id: i32,
            modify_stream: ModifyStream,
        ) -> Result<Option<Stream>, String> {
            let user_id = modify_stream.user_id;

            let stream_opt = self
                .stream_vec
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
                .stream_vec
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
            let stream_opt = self.stream_vec.iter().find(|stream| stream.id == id);

            #[rustfmt::skip]
            let result = if stream_opt.is_none() { 0 } else { 1 };
            Ok(result)
        }
    }
}
