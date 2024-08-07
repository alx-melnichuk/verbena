use crate::profiles::profile_models::Profile;
use crate::users::user_models::UserRole;

use super::profile_models::CreateProfile;

pub trait ProfileOrm {
    /// Get an entity (profile + user) by ID.
    fn get_profile_user_by_id(&self, user_id: i32) -> Result<Option<Profile>, String>;
    /// Find for an entity (profile) by nickname or email.
    fn find_profile_by_nickname_or_email(
        &self,
        nickname: Option<&str>,
        email: Option<&str>,
    ) -> Result<Option<Profile>, String>;
    /// Add a new entry (profile, user).
    fn create_profile_user(&self, create_profile: CreateProfile) -> Result<Profile, String>;

    // /// Modify an entity (user).
    // fn modify_user(&self, id: i32, modify_user_dto: ModifyUserDto) -> Result<Option<User>, String>;

    /// Delete an entity (profile).
    fn delete_profile(&self, user_id: i32) -> Result<Option<Profile>, String>;
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
    use crate::schema;

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
        /// Get an entity (profile + user) by ID.
        fn get_profile_user_by_id(&self, user_id: i32) -> Result<Option<Profile>, String> {
            // Get a connection from the P2D2 pool.
            let mut conn = self.get_conn()?;

            let query = diesel::sql_query("select * from get_profile_user($1);").bind::<sql_types::Integer, _>(user_id);

            // Run query using Diesel to find user by id (and user_id) and return it.
            let opt_profile = query
                .get_result::<Profile>(&mut conn)
                .optional()
                .map_err(|e| format!("get_profile_by_user_id: {}", e.to_string()))?;

            Ok(opt_profile)
        }
        /// Find for an entity (profile) by nickname or email.
        fn find_profile_by_nickname_or_email(
            &self,
            nickname: Option<&str>,
            email: Option<&str>,
        ) -> Result<Option<Profile>, String> {
            let nickname2 = nickname.unwrap_or(&"".to_string()).to_lowercase(); // #?
            let nickname2_len = nickname2.len();
            let email2 = email.unwrap_or(&"".to_string()).to_lowercase();
            let email2_len = email2.len();

            if nickname2_len == 0 && email2_len == 0 {
                return Ok(None);
            }

            // Get a connection from the P2D2 pool.
            let mut conn = self.get_conn()?;

            let query = diesel::sql_query("select * from find_profile_user_by_nickname_or_email($1, $2);")
                .bind::<sql_types::Text, _>(nickname2)
                .bind::<sql_types::Text, _>(email2);

            // Run query using Diesel to find user by id (and user_id) and return it.
            let opt_profile = query
                .get_result::<Profile>(&mut conn)
                .optional()
                .map_err(|e| format!("find_profile_user_by_nickname_or_email: {}", e.to_string()))?;

            Ok(opt_profile)
        }
        /// Add a new entry (profile, user).
        fn create_profile_user(&self, create_profile: CreateProfile) -> Result<Profile, String> {
            let nickname = create_profile.nickname.to_lowercase(); // #?
            let email = create_profile.email.to_lowercase();
            let password = create_profile.password.clone();
            let role = create_profile.role.unwrap_or(UserRole::User);

            // Get a connection from the P2D2 pool.
            let mut conn = self.get_conn()?;

            let query = diesel::sql_query("select * from create_profile_user($1,$2,$3,$4);")
                .bind::<sql_types::Text, _>(nickname.to_string())
                .bind::<sql_types::Text, _>(email.to_string())
                .bind::<sql_types::Text, _>(password.to_string())
                .bind::<schema::sql_types::UserRole, _>(role);

            // Run a query with Diesel to create a new user and return it.
            let profile_user = query
                .get_result::<Profile>(&mut conn)
                .map_err(|e| format!("create_profile_user: {}", e.to_string()))?;

            Ok(profile_user)
        }
        /// Delete an entity (profile).
        fn delete_profile(&self, user_id: i32) -> Result<Option<Profile>, String> {
            // Get a connection from the P2D2 pool.
            let mut conn = self.get_conn()?;

            let query = diesel::sql_query("select * from delete_profile_user_by_user_id($1);")
                .bind::<sql_types::Integer, _>(user_id);

            // Run query using Diesel to find user by id (and user_id) and return it.
            let opt_profile = query
                .get_result::<Profile>(&mut conn)
                .optional()
                .map_err(|e| format!("delete_profile_user_by_user_id: {}", e.to_string()))?;

            Ok(opt_profile)
        }
    }
}

#[cfg(feature = "mockdata")]
pub mod tests {
    use crate::users::user_orm::tests::USER_ID;

    use super::*;

    #[derive(Debug, Clone)]
    pub struct ProfileOrmApp {
        pub profile_vec: Vec<Profile>,
    }

    impl ProfileOrmApp {
        /// Create a new instance.
        pub fn new() -> Self {
            ProfileOrmApp {
                profile_vec: Vec::new(),
            }
        }
        /// Create a new instance with the specified profile list.
        #[cfg(test)]
        pub fn create(profile_list: &[Profile]) -> Self {
            let mut profile_vec: Vec<Profile> = Vec::new();
            for (idx, profile) in profile_list.iter().enumerate() {
                let delta: i32 = idx.try_into().unwrap();
                let mut profile2 = Profile::new(
                    USER_ID + delta,
                    &profile.nickname.to_lowercase(),
                    &profile.email.to_lowercase(),
                    profile.role.clone(),
                    profile.avatar.as_deref(),
                    Some(profile.descript.as_str()),
                    Some(profile.theme.as_str()),
                );
                profile2.created_at = profile.created_at;
                profile2.updated_at = profile.updated_at;
                profile_vec.push(profile2);
            }
            ProfileOrmApp { profile_vec }
        }
        /// Create a new entity instance.
        pub fn new_profile(user_id: i32, nickname: &str, email: &str, role: UserRole) -> Profile {
            Profile::new(
                user_id,
                &nickname.to_lowercase(),
                &email.to_lowercase(),
                role.clone(),
                None,
                None,
                None,
            )
        }
    }

    impl ProfileOrm for ProfileOrmApp {
        /// Get an entity (profile + user) by ID.
        fn get_profile_user_by_id(&self, user_id: i32) -> Result<Option<Profile>, String> {
            let result = self
                .profile_vec
                .iter()
                .find(|profile_user| profile_user.user_id == user_id)
                .map(|profile_user| profile_user.clone());
            Ok(result)
        }
        /// Find for an entity (profile) by nickname or email.
        fn find_profile_by_nickname_or_email(
            &self,
            nickname: Option<&str>,
            email: Option<&str>,
        ) -> Result<Option<Profile>, String> {
            let nickname2 = nickname.unwrap_or(&"".to_string()).to_lowercase();
            let nickname2_len = nickname2.len();
            let email2 = email.unwrap_or(&"".to_string()).to_lowercase();
            let email2_len = email2.len();

            if nickname2_len == 0 && email2_len == 0 {
                return Ok(None);
            }

            let result = self
                .profile_vec
                .iter()
                .find(|profile| {
                    (nickname2_len > 0 && profile.nickname == nickname2) || (email2_len > 0 && profile.email == email2)
                })
                .map(|user| user.clone());

            Ok(result)
        }
        /// Add a new entry (profile, user).
        fn create_profile_user(&self, create_profile: CreateProfile) -> Result<Profile, String> {
            let nickname = create_profile.nickname.to_lowercase();
            let email = create_profile.email.to_lowercase();

            // Check the availability of the profile by nickname and email.
            let opt_profile = self.find_profile_by_nickname_or_email(Some(&nickname), Some(&email))?;
            if opt_profile.is_some() {
                return Err("Profile already exists".to_string());
            }

            let idx: i32 = self.profile_vec.len().try_into().unwrap();
            let user_id: i32 = USER_ID + idx;

            let profile_user = Profile::new(
                user_id,
                &nickname,
                &email,
                create_profile.role.unwrap_or(UserRole::User),
                create_profile.avatar.as_deref(),
                create_profile.descript.as_deref(),
                create_profile.theme.as_deref(),
            );
            Ok(profile_user)
        }
        /// Delete an entity (profile).
        fn delete_profile(&self, user_id: i32) -> Result<Option<Profile>, String> {
            let opt_profile = self.profile_vec.iter().find(|profile| profile.user_id == user_id);

            Ok(opt_profile.map(|u| u.clone()))
        }
    }
}
