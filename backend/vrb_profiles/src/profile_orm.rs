use vrb_authent::user_models::Profile;
use vrb_dbase::dbase::DbPool;

use crate::profile_models::{ModifyUserProfile, UserProfile};

pub trait ProfileOrm {
    /// Get an entity (profile) by ID.
    fn get_profile_by_id(&self, user_id: i32) -> Result<Option<Profile>, String>;

    /// Get an entities  (user, profile) by ID.
    fn get_user_profile_by_id(&self, user_id: i32) -> Result<Option<UserProfile>, String>;

    /// Modify an entity (profile, user).
    fn modify_user_profile(&self, user_id: i32, modify_profile: ModifyUserProfile) -> Result<Option<UserProfile>, String>;

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
    use log::{Level::Info, info, log_enabled};
    use vrb_authent::user_models::Profile;
    use vrb_dbase::{dbase, schema};

    use crate::profile_models::{ModifyUserProfile, StreamLogo, UserProfile};

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
        /// Get an entity (profile) by ID.
        fn get_profile_by_id(&self, user_id: i32) -> Result<Option<Profile>, String> {
            let timer = if log_enabled!(Info) { Some(tm::now()) } else { None };
            // Get a connection from the P2D2 pool.
            let mut conn = self.get_conn()?;
            // Run query using Diesel to find user by id and return it.
            let opt_profile: Option<Profile> = schema::profiles::table
                .filter(schema::profiles::dsl::user_id.eq(user_id))
                .first::<Profile>(&mut conn)
                .optional()
                .map_err(|e| format!("get_profile_by_id: {}", e.to_string()))?;

            if let Some(timer) = timer {
                info!("get_profile_by_id() time: {}", format!("{:.2?}", timer.elapsed()));
            }
            Ok(opt_profile)
        }

        /// Get an entities (user, profile) by ID.
        fn get_user_profile_by_id(&self, user_id: i32) -> Result<Option<UserProfile>, String> {
            let timer = if log_enabled!(Info) { Some(tm::now()) } else { None };
            // Get a connection from the P2D2 pool.
            let mut conn = self.get_conn()?;

            let query = diesel::sql_query("select * from get_user_profile_by_id($1);").bind::<sql_types::Integer, _>(user_id); // $1

            // Run a query with Diesel to create a new user and return it.
            let opt_user_profile = query
                .get_result::<UserProfile>(&mut conn)
                .optional()
                .map_err(|e| format!("get_user_profile_by_id: {}", e.to_string()))?;

            if let Some(timer) = timer {
                info!("get_user_profile_by_id() time: {}", format!("{:.2?}", timer.elapsed()));
            }
            Ok(opt_user_profile)
        }

        /// Modify an entity (user, profile).
        fn modify_user_profile(&self, user_id: i32, modify_profile: ModifyUserProfile) -> Result<Option<UserProfile>, String> {
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

            let query = diesel::sql_query("select * from modify_user_profile($1,$2,$3,$4,$5,$6,$7,$8,$9);")
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
            let opt_user_profile = query
                .get_result::<UserProfile>(&mut conn)
                .optional()
                .map_err(|e| format!("modify_profile_user: {}", e.to_string()))?;

            if let Some(timer) = timer {
                info!("modify_profile() time: {}", format!("{:.2?}", timer.elapsed()));
            }
            Ok(opt_user_profile)
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
    use vrb_authent::{
        config_jwt,
        user_models::{Profile, Session, User},
        user_orm::tests::USER1_ID,
    };
    use vrb_common::consts;

    use crate::{
        config_prfl,
        profile_models::{ModifyUserProfile, UserProfile},
        profile_orm::ProfileOrm,
    };

    #[derive(Debug, Clone)]
    pub struct ProfileOrmApp {
        pub user_profile_vec: Vec<UserProfile>,
        pub session_vec: Vec<Session>,
    }

    impl ProfileOrmApp {
        /// Create a new instance.
        pub fn new() -> Self {
            ProfileOrmApp {
                user_profile_vec: Vec::new(),
                session_vec: Vec::new(),
            }
        }
        /// Create a new instance with the specified profile list.
        /// Sessions are taken from "sessions", if it is empty, they are created automatically.
        pub fn create(user_profiles: &[UserProfile]) -> Self {
            ProfileOrmApp {
                user_profile_vec: user_profiles.to_vec(),
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
        /// Get an entity (profile) by ID.
        fn get_profile_by_id(&self, user_id: i32) -> Result<Option<Profile>, String> {
            let opt_user_profile = self
                .user_profile_vec
                .iter()
                .find(|profile| profile.user_id == user_id)
                .map(|profile| profile.clone());

            let opt_profile = opt_user_profile.map(|user_profile| Profile {
                user_id: user_profile.user_id,
                avatar: user_profile.avatar,
                descript: user_profile.descript,
                theme: user_profile.theme,
                locale: user_profile.locale,
                created_at: user_profile.created_at,
                updated_at: user_profile.updated_at,
            });
            Ok(opt_profile)
        }

        /// Get an entities (user, profile) by ID.
        fn get_user_profile_by_id(&self, user_id: i32) -> Result<Option<UserProfile>, String> {
            let opt_user_profile = self
                .user_profile_vec
                .iter()
                .find(|profile| profile.user_id == user_id)
                .map(|profile| profile.clone());

            Ok(opt_user_profile)
        }

        /// Modify an entity (profile, user).
        fn modify_user_profile(&self, user_id: i32, modify_user_profile: ModifyUserProfile) -> Result<Option<UserProfile>, String> {
            let opt_profile = self.user_profile_vec.iter().find(|profile| (*profile).user_id == user_id);
            let opt_profile3: Option<UserProfile> = if let Some(profile) = opt_profile {
                let profile2 = UserProfile {
                    user_id: profile.user_id,
                    nickname: modify_user_profile.nickname.unwrap_or(profile.nickname.clone()),
                    email: modify_user_profile.email.unwrap_or(profile.email.clone()),
                    role: modify_user_profile.role.unwrap_or(profile.role.clone()),
                    avatar: modify_user_profile.avatar.unwrap_or(profile.avatar.clone()),
                    descript: modify_user_profile.descript.or(profile.descript.clone()),
                    theme: modify_user_profile.theme.or(profile.theme.clone()),
                    locale: modify_user_profile.locale.or(profile.locale.clone()),
                    created_at: profile.created_at,
                    updated_at: Utc::now(),
                };
                Some(profile2)
            } else {
                None
            };
            Ok(opt_profile3)
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
        pub fn profiles(users: &[User]) -> Vec<UserProfile> {
            let profile_vec: Vec<UserProfile> = users.iter().map(|u| UserProfile::from(u.clone())).collect();
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
        pub fn cfg_profile_orm(data_p: Vec<UserProfile>) -> impl FnOnce(&mut web::ServiceConfig) {
            move |config: &mut web::ServiceConfig| {
                let data_user_profile_orm = web::Data::new(ProfileOrmApp::create(&data_p));

                config.app_data(web::Data::clone(&data_user_profile_orm));
            }
        }
    }
}
