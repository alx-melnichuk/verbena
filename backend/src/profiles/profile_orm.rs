use crate::profiles::profile_models::{CreateProfile, ModifyProfile, Profile};

pub trait ProfileOrm {
    /// Get an entity (profile + user) by ID.
    fn get_profile_user_by_id(&self, user_id: i32, is_password: bool) -> Result<Option<Profile>, String>;
    /// Find for an entity (profile) by nickname or email.
    fn find_profile_by_nickname_or_email(
        &self,
        nickname: Option<&str>,
        email: Option<&str>,
        is_password: bool,
    ) -> Result<Option<Profile>, String>;
    /// Add a new entry (profile, user).
    fn create_profile_user(&self, create_profile: CreateProfile) -> Result<Profile, String>;

    /// Modify an entity (profile, user).
    fn modify_profile(&self, user_id: i32, modify_profile: ModifyProfile) -> Result<Option<Profile>, String>;

    /// Delete an entity (profile).
    fn delete_profile(&self, user_id: i32) -> Result<Option<Profile>, String>;
}

pub mod cfg {
    use crate::dbase::DbPool;

    #[cfg(not(all(test, feature = "mockdata")))]
    use super::impls::ProfileOrmApp;
    #[cfg(not(all(test, feature = "mockdata")))]
    pub fn get_profile_orm_app(pool: DbPool) -> ProfileOrmApp {
        ProfileOrmApp::new(pool)
    }

    #[cfg(all(test, feature = "mockdata"))]
    use super::tests::ProfileOrmApp;
    #[cfg(all(test, feature = "mockdata"))]
    pub fn get_profile_orm_app(_: DbPool) -> ProfileOrmApp {
        ProfileOrmApp::new()
    }
}

#[cfg(not(all(test, feature = "mockdata")))]
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
        fn get_profile_user_by_id(&self, user_id: i32, is_password: bool) -> Result<Option<Profile>, String> {
            // Get a connection from the P2D2 pool.
            let mut conn = self.get_conn()?;

            let query = diesel::sql_query("select * from find_profile_user($1, NULL, NULL, $2);")
                .bind::<sql_types::Integer, _>(user_id)
                .bind::<sql_types::Bool, _>(is_password);

            // Run query using Diesel to find user by id (and user_id) and return it.
            let opt_profile = query
                .get_result::<Profile>(&mut conn)
                .optional()
                .map_err(|e| format!("find_profile_user: {}", e.to_string()))?;

            Ok(opt_profile)
        }
        /// Find for an entity (profile) by nickname or email.
        fn find_profile_by_nickname_or_email(
            &self,
            nickname: Option<&str>,
            email: Option<&str>,
            is_password: bool,
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

            let query = diesel::sql_query("select * from find_profile_user(NULL, $1, $2, $3);")
                .bind::<sql_types::Text, _>(nickname2)
                .bind::<sql_types::Text, _>(email2)
                .bind::<sql_types::Bool, _>(is_password);

            // Run query using Diesel to find user by id (and user_id) and return it.
            let opt_profile = query
                .get_result::<Profile>(&mut conn)
                .optional()
                .map_err(|e| format!("find_profile_user: {}", e.to_string()))?;

            Ok(opt_profile)
        }
        /// Add a new entry (profile, user).
        fn create_profile_user(&self, create_profile: CreateProfile) -> Result<Profile, String> {
            let nickname = create_profile.nickname.to_lowercase(); // #?
            let email = create_profile.email.to_lowercase();

            // Get a connection from the P2D2 pool.
            let mut conn = self.get_conn()?;

            let query = diesel::sql_query("select * from create_profile_user($1,$2,$3,$4,$5,$6,$7,$8);")
                .bind::<sql_types::Text, _>(nickname) // $1
                .bind::<sql_types::Text, _>(email) // $2
                .bind::<sql_types::Text, _>(create_profile.password) // $3
                .bind::<sql_types::Nullable<schema::sql_types::UserRole>, _>(create_profile.role) // $4
                .bind::<sql_types::Nullable<sql_types::Text>, _>(create_profile.avatar) // $5
                .bind::<sql_types::Nullable<sql_types::Text>, _>(create_profile.descript) // $6
                .bind::<sql_types::Nullable<sql_types::Text>, _>(create_profile.theme) // $7
                .bind::<sql_types::Nullable<sql_types::Text>, _>(create_profile.locale); // $8

            // Run a query with Diesel to create a new user and return it.
            let profile_user = query
                .get_result::<Profile>(&mut conn)
                .map_err(|e| format!("create_profile_user: {}", e.to_string()))?;

            Ok(profile_user)
        }
        /// Modify an entity (profile, user).
        fn modify_profile(&self, user_id: i32, modify_profile: ModifyProfile) -> Result<Option<Profile>, String> {
            //
            let nickname = modify_profile.nickname.map(|v| v.to_lowercase()); // #?
            let email = modify_profile.email.map(|v| v.to_lowercase());
            let avatar = match modify_profile.avatar {
                Some(value1) => match value1 {
                    Some(value2) => Some(value2),
                    None => Some("".to_string()),
                },
                None => None,
            };

            // Get a connection from the P2D2 pool.
            let mut conn = self.get_conn()?;

            let query = diesel::sql_query("select * from modify_profile_user($1,$2,$3,$4,$5,$6,$7,$8,$9);")
                .bind::<sql_types::Integer, _>(user_id) // $1
                .bind::<sql_types::Nullable<sql_types::Text>, _>(nickname) // $2
                .bind::<sql_types::Nullable<sql_types::Text>, _>(email) // $3
                .bind::<sql_types::Nullable<sql_types::Text>, _>(modify_profile.password) // $4
                .bind::<sql_types::Nullable<schema::sql_types::UserRole>, _>(modify_profile.role) // $5
                .bind::<sql_types::Nullable<sql_types::Text>, _>(avatar) // $6
                .bind::<sql_types::Nullable<sql_types::Text>, _>(modify_profile.descript) // $7
                .bind::<sql_types::Nullable<sql_types::Text>, _>(modify_profile.theme) // $8
                .bind::<sql_types::Nullable<sql_types::Text>, _>(modify_profile.locale); // $9

            // Run a query with Diesel to create a new user and return it.
            let profile_user = query
                .get_result::<Profile>(&mut conn)
                .optional()
                .map_err(|e| format!("modify_profile_user: {}", e.to_string()))?;

            Ok(profile_user)
        }
        /// Delete an entity (profile).
        fn delete_profile(&self, user_id: i32) -> Result<Option<Profile>, String> {
            // Get a connection from the P2D2 pool.
            let mut conn = self.get_conn()?;

            let query =
                diesel::sql_query("select * from delete_profile_user($1);").bind::<sql_types::Integer, _>(user_id);

            // Run query using Diesel to find user by id (and user_id) and return it.
            let opt_profile = query
                .get_result::<Profile>(&mut conn)
                .optional()
                .map_err(|e| format!("delete_profile_user: {}", e.to_string()))?;

            Ok(opt_profile)
        }
    }
}

#[cfg(all(test, feature = "mockdata"))]
pub mod tests {

    use chrono::Utc;

    use super::*;

    use crate::users::user_models::UserRole;

    pub const PROFILE_USER_ID: i32 = 1100;

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
        pub fn create(profile_list: &[Profile]) -> Self {
            let mut profile_vec: Vec<Profile> = Vec::new();
            for (idx, profile) in profile_list.iter().enumerate() {
                let delta: i32 = idx.try_into().unwrap();
                let mut profile2 = Profile::new(
                    PROFILE_USER_ID + delta,
                    &profile.nickname.to_lowercase(),
                    &profile.email.to_lowercase(),
                    profile.role.clone(),
                    profile.avatar.as_deref(),
                    Some(profile.descript.as_str()),
                    Some(profile.theme.as_str()),
                    Some(profile.locale.as_str()),
                );
                profile2.password = profile.password.clone();
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
                None,
            )
        }
    }

    impl ProfileOrm for ProfileOrmApp {
        /// Get an entity (profile + user) by ID.
        fn get_profile_user_by_id(&self, user_id: i32, is_password: bool) -> Result<Option<Profile>, String> {
            let opt_profile = self
                .profile_vec
                .iter()
                .find(|profile_user| profile_user.user_id == user_id)
                .map(|profile_user| profile_user.clone());

            let result = match opt_profile {
                Some(mut profile) if !is_password => {
                    profile.password = "".to_string();
                    Some(profile)
                }
                Some(v) => Some(v),
                None => None,
            };

            Ok(result)
        }
        /// Find for an entity (profile) by nickname or email.
        fn find_profile_by_nickname_or_email(
            &self,
            nickname: Option<&str>,
            email: Option<&str>,
            is_password: bool,
        ) -> Result<Option<Profile>, String> {
            let nickname2 = nickname.unwrap_or(&"".to_string()).to_lowercase();
            let nickname2_len = nickname2.len();
            let email2 = email.unwrap_or(&"".to_string()).to_lowercase();
            let email2_len = email2.len();

            if nickname2_len == 0 && email2_len == 0 {
                return Ok(None);
            }

            let opt_profile = self
                .profile_vec
                .iter()
                .find(|profile| {
                    (nickname2_len > 0 && profile.nickname == nickname2) || (email2_len > 0 && profile.email == email2)
                })
                .map(|user| user.clone());

            let result = match opt_profile {
                Some(mut profile) if !is_password => {
                    profile.password = "".to_string();
                    Some(profile)
                }
                Some(v) => Some(v),
                None => None,
            };

            Ok(result)
        }
        /// Add a new entry (profile, user).
        fn create_profile_user(&self, create_profile: CreateProfile) -> Result<Profile, String> {
            let nickname = create_profile.nickname.to_lowercase();
            let email = create_profile.email.to_lowercase();

            // Check the availability of the profile by nickname and email.
            let opt_profile = self.find_profile_by_nickname_or_email(Some(&nickname), Some(&email), false)?;
            if opt_profile.is_some() {
                return Err("Profile already exists".to_string());
            }

            let idx: i32 = self.profile_vec.len().try_into().unwrap();
            let user_id: i32 = PROFILE_USER_ID + idx;

            let profile_user = Profile::new(
                user_id,
                &nickname,
                &email,
                create_profile.role.unwrap_or(UserRole::User),
                create_profile.avatar.as_deref(),
                create_profile.descript.as_deref(),
                create_profile.theme.as_deref(),
                create_profile.locale.as_deref(),
            );
            Ok(profile_user)
        }
        /// Modify an entity (profile, user).
        fn modify_profile(&self, user_id: i32, modify_profile: ModifyProfile) -> Result<Option<Profile>, String> {
            let opt_profile = self.profile_vec.iter().find(|profile| (*profile).user_id == user_id);
            let opt_profile3: Option<Profile> = if let Some(profile) = opt_profile {
                let profile2 = Profile {
                    user_id: profile.user_id,
                    nickname: modify_profile.nickname.unwrap_or(profile.nickname.clone()),
                    email: modify_profile.email.unwrap_or(profile.email.clone()),
                    password: modify_profile.password.unwrap_or(profile.password.clone()),
                    role: modify_profile.role.unwrap_or(profile.role.clone()),
                    avatar: modify_profile.avatar.unwrap_or(profile.avatar.clone()),
                    descript: modify_profile.descript.unwrap_or(profile.descript.clone()),
                    theme: modify_profile.theme.unwrap_or(profile.theme.clone()),
                    locale: modify_profile.locale.unwrap_or(profile.locale.clone()),
                    created_at: profile.created_at,
                    updated_at: Utc::now(),
                };
                Some(profile2)
            } else {
                None
            };
            Ok(opt_profile3)
        }
        /// Delete an entity (profile).
        fn delete_profile(&self, user_id: i32) -> Result<Option<Profile>, String> {
            let opt_profile = self.profile_vec.iter().find(|profile| profile.user_id == user_id);

            Ok(opt_profile.map(|u| u.clone()))
        }
    }
}
