use super::stream_models::{CreateStream, ModifyStream, Stream};

pub trait StreamOrm {
    /// Find for an entity (stream) by id.
    fn find_stream_by_id(&self, id: i32) -> Result<Option<Stream>, String>;
    /// Find for an entity (stream) by user_id.
    fn find_streams_by_user_id(&self, user_id: i32) -> Result<Vec<Stream>, String>;
    /// Find tag names (stream_tag) by id and user_id.
    fn find_stream_tags(&self, id: i32, user_id: i32) -> Result<Vec<String>, String>;
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

    use diesel::{self, prelude::*, sql_types};
    use diesel::{debug_query, pg::Pg};
    use schema::streams::dsl;

    use crate::dbase;
    use crate::schema;
    use crate::streams::stream_models::{self, CreateStream, ModifyStream, Stream};

    use super::StreamOrm;

    pub const CONN_POOL: &str = "ConnectionPool";
    pub const MAX_LIMIT_STREAM_TAGS: i64 = stream_models::TAG_NAME_MAX_AMOUNT as i64;

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
            let result = schema::streams::table
                .filter(dsl::id.eq(id))
                .first::<Stream>(&mut conn)
                .optional()
                .map_err(|e| format!("find_stream_by_id: {}", e.to_string()))?;

            let query = schema::streams::table.filter(dsl::id.eq(id));
            let query_sql = debug_query::<Pg, _>(&query).to_string();
            eprintln!("query_sql: {}", query_sql);

            Ok(result)
        }
        /// Find for an entity (stream) by user_id.
        fn find_streams_by_user_id(&self, user_id: i32) -> Result<Vec<Stream>, String> {
            // Get a connection from the P2D2 pool.
            let mut conn = self.get_conn()?;
            // Run query using Diesel to find user by user_id and return it (where final_date > now).
            let result = schema::streams::table
                .filter(dsl::user_id.eq(user_id))
                .load::<Stream>(&mut conn)
                .map_err(|e| format!("find_streams_by_user_id: {}", e.to_string()))?;

            Ok(result)
        }

        /// Find tag names (stream_tag) by id and user_id.
        fn find_stream_tags(&self, id: i32, user_id: i32) -> Result<Vec<String>, String> {
            // Get a connection from the P2D2 pool.
            let mut conn = self.get_conn()?;

            let query = diesel::sql_query(
                "SELECT \"name\" FROM get_stream_tags_by_streams($1, $2) LIMIT($3)",
            )
            .bind::<sql_types::Integer, _>(id)
            .bind::<sql_types::Integer, _>(user_id)
            .bind::<sql_types::BigInt, _>(MAX_LIMIT_STREAM_TAGS);

            let result = query
                .get_results::<stream_models::StreamTagName>(&mut conn)
                .map_err(|e| format!("find_stream_tags: {}", e.to_string()))?;

            let result: Vec<String> = result.into_iter().map(|tag_name| tag_name.name).collect();

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
        /// Find for an entity (stream) by id.
        fn find_stream_by_id(&self, id: i32) -> Result<Option<Stream>, String> {
            eprintln!("find_stream_by_id(id: {})", id);
            let result = self
                .stream_vec
                .iter()
                .find(|stream| stream.id == id)
                .map(|stream| stream.clone());
            eprintln!("find_stream_by_id(): {:?}", &result);
            Ok(result)
        }
        /// Find for an entity (stream) by user_id.
        fn find_streams_by_user_id(&self, user_id: i32) -> Result<Vec<Stream>, String> {
            let result = self
                .stream_vec
                .iter()
                .filter(|stream| stream.user_id == user_id)
                .map(|stream| stream.clone())
                .collect();
            Ok(result)
        }
        /// Find tag names (stream_tag) by id and user_id.
        fn find_stream_tags(&self, id: i32, user_id: i32) -> Result<Vec<String>, String> {
            let index_opt = self
                .stream_vec
                .iter()
                .position(|stream| stream.id == id && stream.user_id == user_id);

            if let Some(index) = index_opt {
                let names = self.tags_vec.get(index).unwrap();
                let result: Vec<String> = names.split(',').map(|name| name.to_string()).collect();
                Ok(result)
            } else {
                Err(format!("Not found id: {}, user_id: {}", id, user_id).to_string())
            }
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
