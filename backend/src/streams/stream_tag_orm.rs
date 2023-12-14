use super::stream_models::{CreateStreamTagDto, ModifyStreamTagDto, StreamTag};

pub trait StreamTagOrm {
    // /// Find for an entity (stream_tag) by id.
    // fn find_stream_by_id(&self, id: i32) -> Result<Option<Stream>, String>;
    // /// Find for an entity (stream_tag) by user_id.
    // fn find_stream_by_user_id(&self, user_id: i32) -> Result<Option<Stream>, String>;
    /// Find for an entity (stream_tag) by user_id and stream_id.
    fn find_stream_tag_by_user_id_stream_id(
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

    use crate::dbase;
    use crate::schema;
    use crate::streams::stream_models::{CreateStreamTagDto, ModifyStreamTagDto, StreamTag};

    use super::StreamTagOrm;

    pub const CONN_POOL: &str = "ConnectionPool";
    pub const DB_STREAM_TAG: &str = "Db_StreamTag";
    // pub const MAX_LIMIT_STREAM_TAGS: i64 = 32;

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
        fn find_stream_tag_by_user_id_stream_id(
            &self,
            user_id: i32,
            stream_id: i32,
        ) -> Result<Vec<StreamTag>, String> {
            // use crate::schema::link_stream_tags_to_streams::dsl as dsl_link;

            // Get a connection from the P2D2 pool.
            let mut conn = self.get_conn()?;

            // Run query using Diesel to find user by id and return it.
            /*let result = schema::stream_tags::table
                .filter(dsl::user_id.eq(user_id))
                .limit(MAX_LIMIT_STREAM_TAGS)
                .load(&mut conn)
                .map_err(|e| format!("{}: {}", DB_STREAM_TAG, e.to_string()))?;
            */
            /*
                use schema::*;

                joinable!(posts -> users (user_id));
                allow_tables_to_appear_in_same_query!(posts, users);

                let implicit_on_clause = users::table.inner_join(posts::table);
                let implicit_on_clause_sql = diesel::debug_query::<DB, _>(&implicit_on_clause).to_string();

                joinable!(link_stream_tags_to_streams -> stream_tags (stream_tag_id));
                joinable!(link_stream_tags_to_streams -> streams (stream_id));
            */
            let query2 =
                schema::stream_tags::table.inner_join(schema::link_stream_tags_to_streams::table);

            let query2_sql = debug_query::<Pg, _>(&query2).to_string();
            eprintln!("query2_sql: {}", query2_sql);

            let query3 = query2
                .filter(
                    schema::stream_tags::dsl::user_id
                        .eq(user_id)
                        .and(schema::link_stream_tags_to_streams::dsl::stream_id.eq(stream_id)),
                )
                .select(schema::stream_tags::all_columns)
                .limit(32);

            let query3_sql = debug_query::<Pg, _>(&query3).to_string();
            eprintln!("query3_sql: {}", query3_sql);

            let result2: Vec<StreamTag> = query3
                .load(&mut conn)
                .map_err(|e| format!("{}: {}", DB_STREAM_TAG, e.to_string()))?;
            eprintln!("##result2.len(): {}", result2.len());
            eprintln!("##result2: {:?}", result2);

            /*let result2: Vec<stream_models::LinkStreamTagsToStreams> =
                schema::link_stream_tags_to_streams::table
                    .filter(dsl_link::stream_id.eq(stream_id))
                    // .limit(MAX_LIMIT_STREAM_TAGS)
                    // .load(&mut conn)
                    // .first::<stream_models::LinkStreamTagsToStreams>(&mut conn)
                    .select(schema::link_stream_tags_to_streams::all_columns)
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
