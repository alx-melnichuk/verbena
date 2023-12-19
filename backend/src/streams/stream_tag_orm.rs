use super::stream_models::{CreateStreamTagDto, ModifyStreamTagDto, StreamTag};

pub trait StreamTagOrm {
    /// Find for an entity (stream_tag) by user_id and stream_id.
    fn find_streamtags_by_userid_streamid(
        &self,
        user_id: i32,
        stream_id: i32,
    ) -> Result<Vec<StreamTag>, String>;
    /// Add a new entity (stream_tag).
    fn create_stream_tag(
        &self,
        create_stream_tag_dto: &CreateStreamTagDto,
    ) -> Result<StreamTag, String>;
    /// Modify an entity (stream_tag).
    fn modify_stream_tag(
        &self,
        id: i32,
        modify_stream_tag_dto: &ModifyStreamTagDto,
    ) -> Result<Option<StreamTag>, String>;
    /// Delete an entity (stream_tag).
    fn delete_stream_tag(&self, id: i32) -> Result<usize, String>;
}

pub mod cfg {
    use crate::dbase::DbPool;

    #[cfg(not(feature = "mockdata"))]
    use super::inst::StreamTagOrmApp;
    #[cfg(not(feature = "mockdata"))]
    pub fn get_stream_tag_orm_app(pool: DbPool) -> StreamTagOrmApp {
        StreamTagOrmApp::new(pool)
    }

    #[cfg(feature = "mockdata")]
    use super::tests::StreamTagOrmApp;
    #[cfg(feature = "mockdata")]
    pub fn get_stream_tag_orm_app(_: DbPool) -> StreamTagOrmApp {
        StreamTagOrmApp::new()
    }
}

#[cfg(not(feature = "mockdata"))]
pub mod inst {

    use diesel::{self, prelude::*};
    use diesel::{debug_query, pg::Pg};
    use schema::stream_tags::dsl;
    // use serde_json::to_string;

    use crate::dbase;
    use crate::schema::{self, link_stream_tags_to_streams as link_stream_tags};
    use crate::streams::stream_models::{self, CreateStreamTagDto, ModifyStreamTagDto, StreamTag};

    use super::StreamTagOrm;

    pub const CONN_POOL: &str = "ConnectionPool";
    pub const DB_STREAM_TAG: &str = "Db_StreamTag";
    pub const MAX_LIMIT_STREAM_TAGS: i64 = stream_models::TAG_NAME_MAX_AMOUNT as i64;

    #[derive(Debug, Clone)]
    pub struct StreamTagOrmApp {
        pub pool: dbase::DbPool,
    }

    impl StreamTagOrmApp {
        pub fn new(pool: dbase::DbPool) -> Self {
            StreamTagOrmApp { pool }
        }
        pub fn get_conn(&self) -> Result<dbase::DbPooledConnection, String> {
            (&self.pool).get().map_err(|e| format!("{CONN_POOL}: {}", e.to_string()))
        }
    }

    impl StreamTagOrm for StreamTagOrmApp {
        /// Find for an entity (stream_tag) by user_id and stream_id.
        fn find_streamtags_by_userid_streamid(
            &self,
            user_id: i32,
            stream_id: i32,
        ) -> Result<Vec<StreamTag>, String> {
            // Get a connection from the P2D2 pool.
            let mut conn = self.get_conn()?;

            // Run query using Diesel to find user by id and return it.
            /*let result = schema::stream_tags::table
                .filter(dsl::user_id.eq(user_id))
                .limit(MAX_LIMIT_STREAM_TAGS)
                .load(&mut conn)
                .map_err(|e| format!("{}: {}", DB_STREAM_TAG, e.to_string()))?;
            */
            let query2 = schema::stream_tags::table.inner_join(link_stream_tags::table);

            let query2_sql = debug_query::<Pg, _>(&query2).to_string();
            eprintln!("query2_sql: {}", query2_sql);

            let query3 = query2
                .filter(
                    schema::stream_tags::dsl::user_id
                        .eq(user_id)
                        .and(link_stream_tags::dsl::stream_id.eq(stream_id)),
                )
                .select(schema::stream_tags::all_columns)
                .limit(MAX_LIMIT_STREAM_TAGS);

            let query3_sql = debug_query::<Pg, _>(&query3).to_string();
            eprintln!("query3_sql: {}", query3_sql);

            let result2: Vec<StreamTag> = query3
                .load(&mut conn)
                .map_err(|e| format!("{}: {}", DB_STREAM_TAG, e.to_string()))?;
            eprintln!("##result2.len(): {}", result2.len());
            eprintln!("##result2: {:?}", result2);

            /*let result2: Vec<stream_models::LinkStreamTagsToStreams> =
                link_stream_tags::table
                    .filter(dsl_link::stream_id.eq(stream_id))
                    // .limit(MAX_LIMIT_STREAM_TAGS)
                    // .load(&mut conn)
                    // .first::<stream_models::LinkStreamTagsToStreams>(&mut conn)
                    .select(link_stream_tags::all_columns)
                    .limit(32)
                    .load(&mut conn)
                    .map_err(|e| format!("{}: {}", DB_STREAM_TAG, e.to_string()))?;
            */

            Ok(result2)
        }

        /// Add a new entity (stream_tag).
        fn create_stream_tag(
            &self,
            create_stream_tag_dto: &CreateStreamTagDto,
        ) -> Result<StreamTag, String> {
            // Get a connection from the P2D2 pool.
            let mut conn = self.get_conn()?;
            // Run query using Diesel to add a new user entry.
            let stream_tag: StreamTag = diesel::insert_into(schema::stream_tags::table)
                .values(create_stream_tag_dto)
                .returning(StreamTag::as_returning())
                .get_result(&mut conn)
                .map_err(|e| format!("{}: {}", DB_STREAM_TAG, e.to_string()))?;

            Ok(stream_tag)
        }

        /// Modify an entity (stream_tag).
        fn modify_stream_tag(
            &self,
            id: i32,
            modify_stream_tag_dto: &ModifyStreamTagDto,
        ) -> Result<Option<StreamTag>, String> {
            // Get a connection from the P2D2 pool.
            let mut conn = self.get_conn()?;
            // Run query using Diesel to full or partially modify the user entry.
            let result = diesel::update(dsl::stream_tags.find(id))
                .set(&*modify_stream_tag_dto)
                .returning(StreamTag::as_returning())
                .get_result(&mut conn)
                .optional()
                .map_err(|e| format!("{}: {}", DB_STREAM_TAG, e.to_string()))?;

            Ok(result)
        }

        /// Delete an entity (stream_tag).
        fn delete_stream_tag(&self, id: i32) -> Result<usize, String> {
            // Get a connection from the P2D2 pool.
            let mut conn = self.get_conn()?;
            // Run query using Diesel to delete a entry (stream).
            let count: usize = diesel::delete(dsl::stream_tags.find(id))
                .execute(&mut conn)
                .map_err(|e| format!("{}: {}", DB_STREAM_TAG, e.to_string()))?;

            Ok(count)
        }
    }
}
