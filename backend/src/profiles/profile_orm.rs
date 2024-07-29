use crate::dbase;

use super::profile_models::ProfileUser;

pub trait ProfileOrm {
    /// Find for an entity (profile + user) by user_id.
    fn get_profile_by_user_id(
        &self,
        conn: &mut dbase::DbPooledConnection,
        user_id: i32,
    ) -> Result<Vec<ProfileUser>, diesel::result::Error>;
    // /// Find for an entity (profile) by user_id.
    // fn find_profile_by_user_id(&self, user_id: i32) -> Result<Option<Profile>, String>;

    // /// Add a new entity (user).
    // fn create_user(&self, create_user_dto: CreateUserDto) -> Result<User, String>;

    // /// Modify an entity (user).
    // fn modify_user(&self, id: i32, modify_user_dto: ModifyUserDto) -> Result<Option<User>, String>;

    // /// Delete an entity (user).
    // fn delete_user(&self, id: i32) -> Result<Option<User>, String>;
}

pub mod cfg {
    use crate::dbase::DbPool;

    #[cfg(not(feature = "mockdata"))]
    use super::impls::ProfileOrmApp;
    #[cfg(not(feature = "mockdata"))]
    pub fn get_profile_orm_app(pool: DbPool) -> ProfileOrmApp {
        ProfileOrmApp::new(pool)
    }

    #[cfg(feature = "mockdata")]
    use super::tests::ProfileOrmApp;
    #[cfg(feature = "mockdata")]
    pub fn get_profile_orm_app(_: DbPool) -> ProfileOrmApp {
        ProfileOrmApp::new()
    }
}

#[cfg(not(feature = "mockdata"))]
pub mod impls {

    use diesel::{self, prelude::*, sql_types};
    // use schema::streams::dsl as streams_dsl;
    // use diesel::{self, prelude::*};

    // use crate::schema;

    use super::*;

    pub const CONN_POOL: &str = "ConnectionPool";

    #[derive(Debug, Clone)]
    pub struct ProfileOrmApp {
        pub pool: dbase::DbPool,
    }

    impl ProfileOrmApp {
        pub fn new(pool: dbase::DbPool) -> Self {
            ProfileOrmApp { pool }
        }
        pub fn get_conn(&self) -> Result<dbase::DbPooledConnection, String> {
            (&self.pool).get().map_err(|e| format!("{}: {}", CONN_POOL, e.to_string()))
        }
    }

    impl ProfileOrm for ProfileOrmApp {
        /// Find for an entity (profile + user) by user_id.
        fn get_profile_by_user_id(
            &self,
            conn: &mut dbase::DbPooledConnection,
            user_id: i32,
        ) -> Result<Vec<ProfileUser>, diesel::result::Error> {
            let query = diesel::sql_query("select * from get_profile_user($1);").bind::<sql_types::Integer, _>(user_id);

            query.get_results::<ProfileUser>(conn)
        }

        // /// Find for an entity (profile + user) by user_id.
        /*fn find_profile_by_user_id(&self, user_id: i32) -> Result<Option<Profile>, String> {
            // Get a connection from the P2D2 pool.
            let mut conn = self.get_conn()?;
            // Run query using Diesel to find profile by id and return it.
            let profile_opt = schema::profiles::table
                .filter(schema::profiles::dsl::user_id.eq(user_id))
                .first::<Profile>(&mut conn)
                .optional()
                .map_err(|e| format!("find_profile_by_user_id: {}", e.to_string()))?;

            Ok(profile_opt)
        }*/
    }
}
