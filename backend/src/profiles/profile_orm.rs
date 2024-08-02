use crate::profiles::profile_models::{Profile, ProfileUser};

use super::profile_models::CreateProfileUser;

pub trait ProfileOrm {
    /// Find for an entity (profile + user) by user_id.
    fn get_profile_by_user_id(&self, user_id: i32) -> Result<Option<ProfileUser>, String>;

    /// Add a new entry (profile).
    fn create_profile(&self, profile: Profile) -> Result<Profile, String>;

    /// Add a new entry (profile, user).
    fn create_profile_user(&self, create_profile: CreateProfileUser) -> Result<ProfileUser, String>;

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

    use crate::{dbase, schema};

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

            // Run query using Diesel to find user by id (and user_id) and return it.
            let opt_profile_user = query
                .get_result::<ProfileUser>(&mut conn)
                .optional()
                .map_err(|e| format!("get_profile_by_user_id: {}", e.to_string()))?;

            Ok(opt_profile_user)
        }
        /// Add a new entry (profile).
        fn create_profile(&self, profile: Profile) -> Result<Profile, String> {
            // Get a connection from the P2D2 pool.
            let mut conn = self.get_conn()?;
            // Run query using Diesel to add a new entry (profile).
            let result: Profile = diesel::insert_into(schema::profiles::table)
                .values(profile)
                .returning(Profile::as_returning())
                .get_result(&mut conn)
                .map_err(|e| format!("create_profile: {}", e.to_string()))?;

            Ok(result)
        }
        /// Add a new entry (profile, user).
        fn create_profile_user(&self, create_profile: CreateProfileUser) -> Result<ProfileUser, String> {
            let nickname = create_profile.nickname.to_lowercase(); // #?
            let email = create_profile.email.to_lowercase();
            let password = create_profile.password.clone();
            // let role = create_profile.role.unwrap_or(UserRole::User);

            // Get a connection from the P2D2 pool.
            let mut conn = self.get_conn()?;

            let query = diesel::sql_query("select * from create_user6($1,$2,$3);")
                .bind::<sql_types::Text, _>(nickname.to_string())
                .bind::<sql_types::Text, _>(email.to_string())
                .bind::<sql_types::Text, _>(password.to_string())
                // .bind::<sql_types::Text, _>(role.to_str().to_string())
                ;

            // Run a query with Diesel to create a new user and return it.
            let profile_user = query
                .get_result::<ProfileUser>(&mut conn)
                .map_err(|e| format!("create_user: {}", e.to_string()))?;

            eprintln!("create_user6() res: {:?}", &profile_user);

            Ok(profile_user)
        }
    }
}

#[cfg(feature = "mockdata")]
pub mod tests {
    use chrono::{Duration, Utc};

    use crate::users::user_models::UserRole;

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
                let mut profile_user2 = ProfileUser::new(
                    profile_user.user_id,
                    &profile_user.nickname.to_lowercase(),
                    &profile_user.email.to_lowercase(),
                    profile_user.role.clone(),
                    profile_user.avatar.as_deref(),
                    &profile_user.descript,
                    &profile_user.theme,
                );
                profile_user2.created_at = profile_user.created_at;
                profile_user2.updated_at = profile_user.updated_at;
                profile_user_vec.push(profile_user2);
            }
            ProfileOrmApp { profile_user_vec }
        }
        /// Create a new entity instance.
        pub fn new_profile(user_id: i32, avatar: Option<&str>, descript: &str, theme: &str) -> Profile {
            let now = Utc::now();
            let cr_dt = now + Duration::minutes(-10);

            Profile {
                user_id,
                avatar: avatar.map(|v| v.to_string()),
                descript: descript.to_string(),
                theme: theme.to_string(),
                created_at: cr_dt.clone(),
                updated_at: cr_dt.clone(),
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
        /// Add a new entry (profile).
        fn create_profile(&self, profile: Profile) -> Result<Profile, String> {
            let user_id = profile.user_id;
            // Check the availability of the profile by user_id.
            let opt_profile_user = self.get_profile_by_user_id(user_id)?;
            if opt_profile_user.is_some() {
                return Err("Profile already exists".to_string());
            }

            let result = Self::new_profile(
                user_id,
                profile.avatar.as_ref().map(String::as_ref),
                &profile.descript,
                &profile.theme,
            );

            Ok(result)
        }
        /// Add a new entry (profile, user).
        fn create_profile_user(&self, create_profile: CreateProfileUser) -> Result<ProfileUser, String> {
            let profile_user = ProfileUser::new(
                1234,
                &create_profile.nickname.to_lowercase(),
                &create_profile.email.to_lowercase(),
                create_profile.role.unwrap_or(UserRole::User),
                create_profile.avatar.as_deref(),
                &create_profile.descript,
                &create_profile.theme,
            );
            Ok(profile_user)
        }
    }
}
