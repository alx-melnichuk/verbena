use vrb_dbase::dbase::DbPool;

use crate::profile_models::{ModifyProfile, Profile};

pub trait ProfileOrm {
    /// Get an entity (profile + user) by ID.
    fn get_profile_user_by_id(&self, user_id: i32, is_password: bool) -> Result<Option<Profile>, String>;

    /// Find for an entity (profile) by nickname or email.
    #[rustfmt::skip]
    fn find_profile_by_nickname_or_email(
        &self, nickname: Option<&str>, email: Option<&str>, is_password: bool,
    ) -> Result<Option<Profile>, String>;

    /// Modify an entity (profile, user).
    fn modify_profile(&self, user_id: i32, modify_profile: ModifyProfile) -> Result<Option<Profile>, String>;

    /// Delete an entity (profile).
    fn delete_profile(&self, user_id: i32) -> Result<Option<Profile>, String>;

    /// Filter for the list of stream logos by user ID.
    fn filter_stream_logos(&self, user_id: i32) -> Result<Vec<String>, String>;
}

#[cfg(not(all(test, feature = "mockdata")))]
pub fn get_profile_orm_app(pool: DbPool) -> impls::ProfileOrmApp {
    impls::ProfileOrmApp::new(pool)
}
#[cfg(all(test, feature = "mockdata"))]
pub fn get_profile_orm_app(_: DbPool) -> tests::ProfileOrmApp {
    tests::ProfileOrmApp::new()
}

#[cfg(not(all(test, feature = "mockdata")))]
pub mod impls {
    use std::time::Instant as tm;

    use diesel::{self, prelude::*, sql_types};
    use log::{info, log_enabled, Level::Info};
    use vrb_dbase::{dbase, schema};

    use crate::profile_models::{ModifyProfile, Profile, StreamLogo};

    use super::ProfileOrm;

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
            let timer = if log_enabled!(Info) { Some(tm::now()) } else { None };
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

            if let Some(timer) = timer {
                info!("get_profile_user_by_id() time: {}", format!("{:.2?}", timer.elapsed()));
            }
            Ok(opt_profile)
        }

        /// Find for an entity (profile) by nickname or email.
        fn find_profile_by_nickname_or_email(
            &self,
            nickname: Option<&str>,
            email: Option<&str>,
            is_password: bool,
        ) -> Result<Option<Profile>, String> {
            let timer = if log_enabled!(Info) { Some(tm::now()) } else { None };

            let nickname2 = nickname.unwrap_or(&"".to_string()).to_lowercase();
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

            if let Some(timer) = timer {
                #[rustfmt::skip]
                info!("find_profile_by_nickname_or_email() time: {}", format!("{:.2?}", timer.elapsed()));
            }
            Ok(opt_profile)
        }

        /// Modify an entity (profile, user).
        fn modify_profile(&self, user_id: i32, modify_profile: ModifyProfile) -> Result<Option<Profile>, String> {
            let timer = if log_enabled!(Info) { Some(tm::now()) } else { None };

            let nickname = modify_profile.nickname.map(|v| v.to_lowercase());
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

            if let Some(timer) = timer {
                info!("modify_profile() time: {}", format!("{:.2?}", timer.elapsed()));
            }
            Ok(profile_user)
        }

        /// Delete an entity (profile).
        fn delete_profile(&self, user_id: i32) -> Result<Option<Profile>, String> {
            let timer = if log_enabled!(Info) { Some(tm::now()) } else { None };
            // Get a connection from the P2D2 pool.
            let mut conn = self.get_conn()?;
            #[rustfmt::skip]
            let query =
                diesel::sql_query("select * from delete_profile_user($1);").bind::<sql_types::Integer, _>(user_id);

            // Run a query using Diesel to delete the "profile" entity by ID and return the data for that entity.
            let opt_profile = query
                .get_result::<Profile>(&mut conn)
                .optional()
                .map_err(|e| format!("delete_profile_user: {}", e.to_string()))?;

            if let Some(timer) = timer {
                info!("delete_profile() time: {}", format!("{:.2?}", timer.elapsed()));
            }
            Ok(opt_profile)
        }

        /// Filter for the list of stream logos by user ID.
        fn filter_stream_logos(&self, user_id: i32) -> Result<Vec<String>, String> {
            let timer = if log_enabled!(Info) { Some(tm::now()) } else { None };
            // Get a connection from the P2D2 pool.
            let mut conn = self.get_conn()?;

            let query = diesel::sql_query("select logo from filter_streams(null, $1, true, null);")
                .bind::<sql_types::Nullable<sql_types::Integer>, _>(user_id); // $1

            // Run a query using Diesel to find a list of users based on the given parameters.
            let stream_logos: Vec<StreamLogo> = query.load(&mut conn).map_err(|e| format!("filter_streams: {}", e.to_string()))?;

            let result = stream_logos.into_iter().map(|v| v.logo.clone()).collect();

            if let Some(timer) = timer {
                info!("filter_stream_logos() time: {}", format!("{:.2?}", timer.elapsed()));
            }
            Ok(result)
        }
    }
}

#[cfg(all(test, feature = "mockdata"))]
pub mod tests {

    use actix_web::web;
    use chrono::Utc;
    use vrb_authent::{config_jwt, user_models::{User, USER1_ID}};
    use vrb_common::consts;

    use crate::{
        config_prfl,
        profile_models::{self, Profile, Session},
        profile_orm::ProfileOrm,
    };

    #[derive(Debug, Clone)]
    pub struct ProfileOrmApp {
        pub profile_vec: Vec<Profile>,
        pub session_vec: Vec<Session>,
    }

    impl ProfileOrmApp {
        /// Create a new instance.
        pub fn new() -> Self {
            ProfileOrmApp {
                profile_vec: Vec::new(),
                session_vec: Vec::new(),
            }
        }
        /// Create a new instance with the specified profile list.
        /// Sessions are taken from "sessions", if it is empty, they are created automatically.
        pub fn create(profiles: &[Profile]) -> Self {
            ProfileOrmApp {
                profile_vec: profiles.to_vec(),
                session_vec: Vec::new(),
            }
        }
        #[rustfmt::skip]
        pub fn stream_logo_alias(user_id: i32) -> Option<String> {
            let idx = user_id - USER1_ID;
            if -1 < idx && idx < 4 { Some(format!("{}/file_logo_{}.png", consts::ALIAS_LOGO_FILES_DIR, idx)) } else { None }
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
                .find(|profile| (nickname2_len > 0 && profile.nickname == nickname2) || (email2_len > 0 && profile.email == email2))
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

        /// Modify an entity (profile, user).
        fn modify_profile(&self, user_id: i32, modify_profile: profile_models::ModifyProfile) -> Result<Option<Profile>, String> {
            let opt_profile = self.profile_vec.iter().find(|profile| (*profile).user_id == user_id);
            let opt_profile3: Option<Profile> = if let Some(profile) = opt_profile {
                let profile2 = Profile {
                    user_id: profile.user_id,
                    nickname: modify_profile.nickname.unwrap_or(profile.nickname.clone()),
                    email: modify_profile.email.unwrap_or(profile.email.clone()),
                    password: modify_profile.password.unwrap_or(profile.password.clone()),
                    role: modify_profile.role.unwrap_or(profile.role.clone()),
                    avatar: modify_profile.avatar.unwrap_or(profile.avatar.clone()),
                    descript: modify_profile.descript.or(profile.descript.clone()),
                    theme: modify_profile.theme.or(profile.theme.clone()),
                    locale: modify_profile.locale.or(profile.locale.clone()),
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

        /// Filter for the list of stream logos by user ID.
        fn filter_stream_logos(&self, user_id: i32) -> Result<Vec<String>, String> {
            let mut result: Vec<String> = vec![];
            let opt_stream_logo = Self::stream_logo_alias(user_id);
            if opt_stream_logo.is_some() {
                result.push(opt_stream_logo.unwrap().clone());
            }
            Ok(result)
        }
    }

    pub struct ProfileOrmTest {}

    impl ProfileOrmTest {
        pub fn profiles(users: &[User]) -> Vec<Profile> {
            let profile_vec: Vec<Profile> = users.iter().map(|u| Profile::from(u.clone())).collect();
            profile_vec
        }
        pub fn cfg_config_jwt(config_jwt: config_jwt::ConfigJwt) -> impl FnOnce(&mut web::ServiceConfig) {
            move |config: &mut web::ServiceConfig| {
                let data_config_jwt = web::Data::new(config_jwt);
                config.app_data(web::Data::clone(&data_config_jwt));
            }
        }
        pub fn cfg_config_prfl(config_prfl: config_prfl::ConfigPrfl) -> impl FnOnce(&mut web::ServiceConfig) {
            move |config: &mut web::ServiceConfig| {
                let data_config_prfl = web::Data::new(config_prfl);
                config.app_data(web::Data::clone(&data_config_prfl));
            }
        }
        pub fn cfg_profile_orm(data_p: Vec<Profile>) -> impl FnOnce(&mut web::ServiceConfig) {
            move |config: &mut web::ServiceConfig| {
                let data_profile_orm = web::Data::new(ProfileOrmApp::create(&data_p));

                config.app_data(web::Data::clone(&data_profile_orm));
            }
        }
    }
}
