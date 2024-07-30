use crate::profiles::profile_models::ProfileUser;

pub trait ProfileOrm {
    /// Find for an entity (profile + user) by user_id.
    fn get_profile_by_user_id(&self, user_id: i32) -> Result<Option<ProfileUser>, String>;
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

    use crate::dbase;

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
        fn get_profile_by_user_id(&self, user_id: i32) -> Result<Option<ProfileUser>, String> {
            // Get a connection from the P2D2 pool.
            let mut conn = self.get_conn()?;

            let query = diesel::sql_query("select * from get_profile_user($1);").bind::<sql_types::Integer, _>(user_id);

            // query.get_results::<Option<ProfileUser>>(&conn)

            // Run query using Diesel to find user by id (and user_id) and return it.
            let opt_profile_user = query
                .get_result::<ProfileUser>(&mut conn)
                .optional()
                .map_err(|e| format!("get_profile_by_user_id: {}", e.to_string()))?;

            Ok(opt_profile_user)
        }
    }
}

#[cfg(feature = "mockdata")]
pub mod tests {
    use chrono::{Duration, Utc};

    use crate::profiles::profile_models;

    use super::*;

    #[derive(Debug, Clone)]
    pub struct ProfileOrmApp {
        pub profile_user_vec: Vec<ProfileUser>,
    }

    impl ProfileOrmApp {
        /// Create a new instance.
        pub fn new() -> Self {
            ProfileOrmApp {
                profile_user_vec: Vec::new(),
            }
        }
        /// Create a new instance with the specified profile list.
        #[cfg(test)]
        pub fn create(profile_user_list: &[ProfileUser]) -> Self {
            let mut profile_user_vec: Vec<ProfileUser> = Vec::new();
            for profile_user in profile_user_list.iter() {
                profile_user_vec.push(ProfileUser {
                    user_id: profile_user.user_id,
                    nickname: profile_user.nickname.to_lowercase(),
                    email: profile_user.email.to_lowercase(),
                    avatar: profile_user.avatar.clone(),
                    descript: profile_user.descript.to_string(),
                    theme: profile_user.theme.to_string(),
                    created_at: profile_user.created_at,
                    updated_at: profile_user.updated_at,
                });
            }
            ProfileOrmApp { profile_user_vec }
        }
        /// Create a new entity instance.
        pub fn new_profile_user(user_id: i32, nickname: &str, email: &str) -> ProfileUser {
            let now = Utc::now();
            let cr_dt = now + Duration::minutes(-10);

            ProfileUser {
                user_id,
                nickname: nickname.to_lowercase(),
                email: email.to_lowercase(),
                avatar: None,
                descript: profile_models::PROFILE_DESCRIPT_DEF.to_string(),
                theme: profile_models::PROFILE_THEME_LIGHT_DEF.to_string(),
                created_at: cr_dt,
                updated_at: cr_dt,
            }
        }
    }

    impl ProfileOrm for ProfileOrmApp {
        /// Find for an entity (profile + user) by user_id.
        fn get_profile_by_user_id(&self, user_id: i32) -> Result<Option<ProfileUser>, String> {
            let result = self
                .profile_user_vec
                .iter()
                .find(|profile_user| profile_user.user_id == user_id)
                .map(|profile_user| profile_user.clone());
            Ok(result)
        }
    }
}
