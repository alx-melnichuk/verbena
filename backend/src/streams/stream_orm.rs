use super::stream_models::{CreateStreamInfoDto, ModifyStreamInfoDto, Stream};

pub trait StreamOrm {
    /// Find for an entity (stream) by id.
    fn find_stream_by_id(&self, id: i32) -> Result<Option<Stream>, String>;
    /// Find for an entity (stream) by user_id.
    fn find_streams_by_user_id(&self, user_id: i32) -> Result<Vec<Stream>, String>;
    /// Add a new entity (stream).
    fn create_stream(
        &self,
        user_id: i32,
        create_stream_dto: &CreateStreamInfoDto,
    ) -> Result<Stream, String>;
    /// Modify an entity (stream).
    fn modify_stream(
        &self,
        id: i32,
        user_id: i32,
        modify_stream_dto: &ModifyStreamInfoDto,
    ) -> Result<Option<Stream>, String>;
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

    use diesel::{self, prelude::*};
    use schema::streams::dsl;

    use crate::dbase;
    use crate::schema;
    use crate::streams::stream_models::{self, CreateStreamInfoDto, ModifyStreamDto, Stream};

    use super::StreamOrm;

    pub const CONN_POOL: &str = "ConnectionPool";
    pub const DB_STREAM: &str = "Db_Stream";

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
                .map_err(|e| format!("{}: {}", DB_STREAM, e.to_string()))?;

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
                .map_err(|e| format!("{}: {}", DB_STREAM, e.to_string()))?;

            Ok(result)
        }

        /// Add a new entity (stream).
        fn create_stream(
            &self,
            user_id: i32,
            create_stream_info_dto: &CreateStreamInfoDto,
        ) -> Result<Stream, String> {
            // Get a connection from the P2D2 pool.
            let mut conn = self.get_conn()?;
            let mut create_stream =
                stream_models::CreateStream::from(create_stream_info_dto.to_owned());
            create_stream.user_id = user_id;
            // Run query using Diesel to add a new entry (stream).
            let stream = diesel::insert_into(schema::streams::table)
                .values(create_stream)
                .returning(Stream::as_returning())
                .get_result(&mut conn)
                .map_err(|e| format!("{}: {}", DB_STREAM, e.to_string()))?;

            Ok(stream)
        }

        /// Modify an entity (stream).
        fn modify_stream(
            &self,
            id: i32,
            user_id: i32,
            modify_stream_dto: &ModifyStreamInfoDto,
        ) -> Result<Option<Stream>, String> {
            // Get a connection from the P2D2 pool.
            let mut conn = self.get_conn()?;
            // Run query using Diesel to full or partially modify the user entry.
            let result = diesel::update(dsl::streams.find(id))
                .set(&*modify_stream_dto)
                .returning(Stream::as_returning())
                .get_result(&mut conn)
                .optional()
                .map_err(|e| format!("{}: {}", DB_STREAM, e.to_string()))?;

            Ok(result)
        }

        /// Delete an entity (stream).
        fn delete_stream(&self, id: i32) -> Result<usize, String> {
            // Get a connection from the P2D2 pool.
            let mut conn = self.get_conn()?;
            // Run query using Diesel to delete a entry (stream).
            let count: usize = diesel::delete(dsl::streams.find(id))
                .execute(&mut conn)
                .map_err(|e| format!("{}: {}", DB_STREAM, e.to_string()))?;

            Ok(count)
        }
    }
}

#[cfg(feature = "mockdata")]
pub mod tests {
    use chrono::Utc;

    use crate::streams::stream_models::{
        self, CreateStreamInfoDto, ModifyStreamInfoDto, Stream, StreamState,
    };

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
    }

    impl StreamOrmApp {
        /// Create a new instance.
        pub fn new() -> Self {
            StreamOrmApp {
                stream_vec: Vec::new(),
            }
        }
    }

    impl StreamOrm for StreamOrmApp {
        /// Find for an entity (stream) by id.
        fn find_stream_by_id(&self, id: i32) -> Result<Option<Stream>, String> {
            let result = self
                .stream_vec
                .iter()
                .find(|stream| stream.id == id)
                .map(|stream| stream.clone());
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

        /// Add a new entity (stream).
        fn create_stream(
            &self,
            user_id: i32,
            create_stream_info_dto: &CreateStreamInfoDto,
        ) -> Result<Stream, String> {
            let now = Utc::now();
            let idx: i32 = self.stream_vec.len().try_into().unwrap();
            let new_id: i32 = STREAM_ID + idx;

            let mut create_stream =
                stream_models::CreateStream::from(create_stream_info_dto.to_owned());
            create_stream.user_id = user_id;

            let source_copy = create_stream.source.clone();

            let stream_saved = Stream {
                id: new_id,
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
                source: source_copy.unwrap_or(DEF_SOURCE.to_string()),
                created_at: now,
                updated_at: now,
            };
            Ok(stream_saved)
        }

        /// Modify an entity (stream).
        fn modify_stream(
            &self,
            id: i32,
            user_id: i32,
            modify_stream_dto: &ModifyStreamInfoDto,
        ) -> Result<Option<Stream>, String> {
            let stream_opt = self
                .stream_vec
                .iter()
                .find(|stream| stream.id == id && stream.user_id == user_id);
            if stream_opt.is_none() {
                return Ok(None);
            }
            let stream = stream_opt.unwrap();
            let stream_saved = Stream {
                id: stream.id,
                user_id: stream.user_id,
                title: modify_stream_dto.title.to_owned(),
                descript: modify_stream_dto.descript.to_owned(),
                logo: modify_stream_dto.logo.clone(),
                starttime: modify_stream_dto.starttime,
                live: modify_stream_dto.live,
                state: modify_stream_dto.state,
                started: modify_stream_dto.started.clone(),
                stopped: modify_stream_dto.stopped.clone(),
                status: modify_stream_dto.status,
                source: modify_stream_dto.source.to_owned(),
                created_at: stream.created_at,
                updated_at: stream.updated_at,
            };

            Ok(Some(stream_saved))
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
