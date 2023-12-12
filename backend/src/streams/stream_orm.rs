use super::stream_models::{CreateStreamDto, Stream};

pub trait StreamOrm {
    /// Find for an entity (stream) by id.
    fn find_stream_by_id(&self, id: i32) -> Result<Option<Stream>, String>;
    /// Find for an entity (stream) by user_id.
    fn find_stream_by_user_id(&self, user_id: i32) -> Result<Option<Stream>, String>;
    /// Add a new entity (stream).
    fn create_stream(&self, create_stream_dto: &CreateStreamDto) -> Result<Stream, String>;
    /// Modify an entity (stream).
    fn modify_stream(
        &self,
        id: i32,
        modify_stream_dto: &CreateStreamDto,
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
    use crate::streams::stream_models::{CreateStreamDto, Stream};

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
        fn find_stream_by_user_id(&self, user_id: i32) -> Result<Option<Stream>, String> {
            // Get a connection from the P2D2 pool.
            let mut conn = self.get_conn()?;
            // Run query using Diesel to find user by user_id and return it (where final_date > now).
            let result = schema::streams::table
                .filter(dsl::user_id.eq(user_id))
                .first::<Stream>(&mut conn)
                .optional()
                .map_err(|e| format!("{}: {}", DB_STREAM, e.to_string()))?;

            Ok(result)
        }

        /// Add a new entity (stream).
        fn create_stream(&self, create_stream_dto: &CreateStreamDto) -> Result<Stream, String> {
            let create_stream_dto2 = create_stream_dto.clone();

            // Get a connection from the P2D2 pool.
            let mut conn = self.get_conn()?;
            // Run query using Diesel to add a new user entry.
            let stream: Stream = diesel::insert_into(schema::streams::table)
                .values(create_stream_dto2)
                .returning(Stream::as_returning())
                .get_result(&mut conn)
                .map_err(|e| format!("{}: {}", DB_STREAM, e.to_string()))?;

            Ok(stream)
        }

        /// Modify an entity (stream).
        fn modify_stream(
            &self,
            id: i32,
            create_stream_dto: &CreateStreamDto,
        ) -> Result<Option<Stream>, String> {
            let create_stream_dto2 = create_stream_dto.clone();

            // Get a connection from the P2D2 pool.
            let mut conn = self.get_conn()?;
            // Run query using Diesel to full or partially modify the user entry.
            let result = diesel::update(dsl::streams.find(id))
                .set(&create_stream_dto2)
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
