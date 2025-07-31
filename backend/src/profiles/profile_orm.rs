use crate::profiles::profile_models::{CreateProfile, ModifyProfile, Profile, Session};

pub trait ProfileOrm {
    /// Get an entity (profile + user) by ID.
    fn get_profile_user_by_id(&self, user_id: i32, is_password: bool) -> Result<Option<Profile>, String>;

    /// Find for an entity (profile) by nickname or email.
    #[rustfmt::skip]
    fn find_profile_by_nickname_or_email(
        &self, nickname: Option<&str>, email: Option<&str>, is_password: bool,
    ) -> Result<Option<Profile>, String>;

    /// Add a new entry (profile, user).
    fn create_profile_user(&self, create_profile: CreateProfile) -> Result<Profile, String>;

    /// Modify an entity (profile, user).
    fn modify_profile(&self, user_id: i32, modify_profile: ModifyProfile) -> Result<Option<Profile>, String>;

    /// Delete an entity (profile).
    fn delete_profile(&self, user_id: i32) -> Result<Option<Profile>, String>;

    /// Get an entity (session) by ID.
    fn get_session_by_id(&self, user_id: i32) -> Result<Option<Session>, String>;

    /// Modify the entity (session).
    fn modify_session(&self, user_id: i32, num_token: Option<i32>) -> Result<Option<Session>, String>;

    // There is no need to delete the entity (session), since it is deleted cascade when deleting an entry in the users table.

    /// Filter for the list of stream logos by user ID.
    fn filter_stream_logos(&self, user_id: i32) -> Result<Vec<String>, String>;
}

pub mod cfg {
    use vrb_dbase::dbase::DbPool;

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
    use std::time::Instant as tm;

    use diesel::{self, prelude::*, sql_types};
    use log::{info, log_enabled, Level::Info};
    use vrb_dbase::{dbase, schema};

    use crate::profiles::profile_models::StreamLogo;

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

        /// Add a new entry (profile, user).
        fn create_profile_user(&self, create_profile: CreateProfile) -> Result<Profile, String> {
            let timer = if log_enabled!(Info) { Some(tm::now()) } else { None };

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

            if let Some(timer) = timer {
                info!("create_profile_user() time: {}", format!("{:.2?}", timer.elapsed()));
            }
            Ok(profile_user)
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

        /// Get an entity (session) by ID.
        fn get_session_by_id(&self, user_id: i32) -> Result<Option<Session>, String> {
            let timer = if log_enabled!(Info) { Some(tm::now()) } else { None };
            // Get a connection from the P2D2 pool.
            let mut conn = self.get_conn()?;
            // Run query using Diesel to find user by id and return it.
            let opt_session: Option<Session> = schema::sessions::table
                .filter(schema::sessions::dsl::user_id.eq(user_id))
                .first::<Session>(&mut conn)
                .optional()
                .map_err(|e| format!("get_session_by_id: {}", e.to_string()))?;

            if let Some(timer) = timer {
                info!("get_session_by_id() time: {}", format!("{:.2?}", timer.elapsed()));
            }
            Ok(opt_session)
        }

        /// Perform a full or partial change to a session record.
        fn modify_session(&self, user_id: i32, num_token: Option<i32>) -> Result<Option<Session>, String> {
            let timer = if log_enabled!(Info) { Some(tm::now()) } else { None };
            // Get a connection from the P2D2 pool.
            let mut conn = self.get_conn()?;
            // Run query using Diesel to full or partially modify the session entry.
            let result = diesel::update(schema::sessions::dsl::sessions.find(user_id))
                .set(schema::sessions::dsl::num_token.eq(num_token))
                .returning(Session::as_returning())
                .get_result(&mut conn)
                .optional()
                .map_err(|e| format!("modify_session: {}", e.to_string()))?;

            if let Some(timer) = timer {
                info!("modify_session() time: {}", format!("{:.2?}", timer.elapsed()));
            }
            Ok(result)
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

    use actix_web::{http, web};
    use chrono::Utc;
    use vrb_dbase::db_enums::UserRole;
    use vrb_tools::{consts, token_coding, token_data::BEARER};

    use crate::profiles::{config_jwt, config_prfl};

    use super::*;

    pub const ADMIN: u8 = 0;
    pub const USER: u8 = 1;
    pub const PROFILE_USER_ID: i32 = 1100;
    pub const NUM_TOKEN_USER1: i32 = 1234;
    pub const PROFILE_USER_ID_NO_SESSION: i32 = 1199;

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
        pub fn create(profiles: &[Profile], sessions: &[Session]) -> Self {
            let mut profile_vec: Vec<Profile> = Vec::new();
            let mut session_vec: Vec<Session> = Vec::new();
            for (idx, profile) in profiles.iter().enumerate() {
                let is_no_session = profile.user_id == PROFILE_USER_ID_NO_SESSION;
                let delta: i32 = idx.try_into().unwrap();
                let user_id = if is_no_session { profile.user_id } else { PROFILE_USER_ID + delta };
                let mut profile2 = Profile::new(
                    user_id,
                    &profile.nickname.to_lowercase(),
                    &profile.email.to_lowercase(),
                    profile.role.clone(),
                    profile.avatar.as_deref(),
                    profile.descript.as_deref(),
                    profile.theme.as_deref(),
                    profile.locale.as_deref(),
                );
                profile2.password = profile.password.clone();
                profile2.created_at = profile.created_at;
                profile2.updated_at = profile.updated_at;
                profile_vec.push(profile2);

                let opt_session = sessions.iter().find(|v| (*v).user_id == profile.user_id);
                if let Some(session) = opt_session {
                    #[rustfmt::skip]
                    session_vec.push(Session { user_id, num_token: session.num_token });
                } else if !is_no_session {
                    session_vec.push(Session { user_id, num_token: None });
                }
            }
            ProfileOrmApp { profile_vec, session_vec }
        }
        /// Create a new instance of the Profile entity.
        pub fn new_profile(user_id: i32, nickname: &str, email: &str, role: UserRole) -> Profile {
            Profile::new(user_id, &nickname.to_lowercase(), &email.to_lowercase(), role.clone(), None, None, None, None)
        }
        /// Create a new instance of the Session entity.
        pub fn new_session(user_id: i32, num_token: Option<i32>) -> Session {
            Session { user_id, num_token }
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

        /// Get an entity (session) by ID.
        fn get_session_by_id(&self, user_id: i32) -> Result<Option<Session>, String> {
            let opt_session: Option<Session> = self
                .session_vec
                .iter()
                .find(|session| session.user_id == user_id)
                .map(|session| session.clone());

            Ok(opt_session)
        }
        
        /// Modify the entity (session).
        fn modify_session(&self, user_id: i32, num_token: Option<i32>) -> Result<Option<Session>, String> {
            let opt_session: Option<Session> = self.get_session_by_id(user_id)?;
            if opt_session.is_none() {
                return Ok(None);
            }
            let mut res_session = opt_session.unwrap();
            let new_session = Session { user_id, num_token };
            res_session.num_token = new_session.num_token;

            Ok(Some(res_session))
        }

        /// Filter for the list of stream logos by user ID.
        fn filter_stream_logos(&self, user_id: i32) -> Result<Vec<String>, String> {
            let mut result: Vec<String> = vec![];
            let opt_stream_logo = ProfileOrmTest::stream_logo_alias(user_id);
            if opt_stream_logo.is_some() {
                result.push(opt_stream_logo.unwrap().clone());
            }
            Ok(result)
        }
    }

    pub struct ProfileOrmTest {}

    impl ProfileOrmTest {
        pub fn user_ids() -> Vec<i32> {
            vec![PROFILE_USER_ID + 0, PROFILE_USER_ID + 1, PROFILE_USER_ID + 2, PROFILE_USER_ID + 3]
        }
        pub fn user_names() -> Vec<String> {
            vec![
                "oliver_taylor".to_string(),
                "robert_brown".to_string(),
                "mary_williams".to_string(),
                "ava_wilson".to_string(),
            ]
        }
        // =v ?
        pub fn stream_ids() -> Vec<i32> {
            vec![
                1, // Owner user idx 0 (live: true)  1100 oliver_taylor
                2, // Owner user idx 1 (live: true)  1101 robert_brown
                3, // Owner user idx 2 (live: false) 1102 mary_williams
                4, // Owner user idx 3  blocked      1103 ava_wilson
            ]
        }
        // =v ?
        #[rustfmt::skip]
        pub fn stream_logo_alias(user_id: i32) -> Option<String> {
            let idx = user_id - PROFILE_USER_ID;
            if -1 < idx && idx < 4 { Some(format!("{}/file_logo_{}.png", consts::ALIAS_LOGO_FILES_DIR, idx)) } else { None }
        }
        // =v ?
        #[rustfmt::skip]
        pub fn stream_logo_path(user_id: i32) -> Option<String> {
            let idx = user_id - PROFILE_USER_ID;
            if -1 < idx && idx < 4 { Some(format!("{}/file_logo_{}.png", consts::LOGO_FILES_DIR, idx)) } else { None }
        }
        pub fn header_auth(token: &str) -> (http::header::HeaderName, http::header::HeaderValue) {
            let header_value = http::header::HeaderValue::from_str(&format!("{}{}", BEARER, token)).unwrap();
            (http::header::AUTHORIZATION, header_value)
        }
        
        pub fn profiles(roles: &[u8]) -> (Vec<Profile>, Vec<Session>) {
            let mut profile_vec: Vec<Profile> = Vec::new();
            let mut session_vec: Vec<Session> = Vec::new();
            let user_ids = ProfileOrmTest::user_ids();
            let user_names = ProfileOrmTest::user_names();
            let len = if roles.len() > user_ids.len() { user_ids.len() } else { roles.len() };
            for index in 0..len {
                let user_id = user_ids.get(index).unwrap().clone();
                let nickname = user_names.get(index).unwrap().clone().to_lowercase();
                let email = format!("{}@gmail.com", nickname);
                #[rustfmt::skip]
                let role = if roles.get(index).unwrap().clone() == ADMIN { UserRole::Admin } else { UserRole::User };

                let profile = Profile::new(user_id, &nickname, &email, role, None, None, None, None);
                profile_vec.push(profile);
                let num_token = if user_id == PROFILE_USER_ID { Some(NUM_TOKEN_USER1) } else { None };
                session_vec.push(Session { user_id, num_token });
            }
            let profile_orm_app = ProfileOrmApp { profile_vec, session_vec };

            (profile_orm_app.profile_vec, profile_orm_app.session_vec)
        }
        pub fn cfg() -> (config_jwt::ConfigJwt, config_prfl::ConfigPrfl) {
            let config_jwt = config_jwt::get_test_config();
            let config_prfl = config_prfl::get_test_config();
            (config_jwt, config_prfl)
        }
        pub fn token1() -> String {
            let config_jwt = config_jwt::get_test_config();
            let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();
            token_coding::encode_token(PROFILE_USER_ID, NUM_TOKEN_USER1, &jwt_secret, config_jwt.jwt_access).unwrap()
        }

        pub fn cfg_config_jwt(config_jwt: config_jwt::ConfigJwt) -> impl FnOnce(&mut web::ServiceConfig) {
            move |config: &mut web::ServiceConfig| {
                let data_config_jwt = web::Data::new(config_jwt);
                config
                .app_data(web::Data::clone(&data_config_jwt));
            }
        }
        pub fn cfg_config_prfl(config_prfl: config_prfl::ConfigPrfl) -> impl FnOnce(&mut web::ServiceConfig) {
            move |config: &mut web::ServiceConfig| {
                let data_config_prfl = web::Data::new(config_prfl);
                config
                .app_data(web::Data::clone(&data_config_prfl));
            }
        }
        pub fn cfg_profile_orm(data_p: (Vec<Profile>, Vec<Session>)) -> impl FnOnce(&mut web::ServiceConfig) {
            move |config: &mut web::ServiceConfig| {
                let data_profile_orm = web::Data::new(
                    ProfileOrmApp::create(&data_p.0, &data_p.1));

                config.app_data(web::Data::clone(&data_profile_orm));
            }
        }
        // =v ?
        pub fn config(
            cfg_p: (config_jwt::ConfigJwt, config_prfl::ConfigPrfl), // configuration
            data_p: (Vec<Profile>, Vec<Session>),                    // data vectors
        ) -> impl FnOnce(&mut web::ServiceConfig) {
            move |config: &mut web::ServiceConfig| {
                let data_config_jwt = web::Data::new(cfg_p.0);
                let data_config_prfl = web::Data::new(cfg_p.1);

                let data_profile_orm = web::Data::new(ProfileOrmApp::create(&data_p.0, &data_p.1));

                config
                    .app_data(web::Data::clone(&data_config_jwt))
                    .app_data(web::Data::clone(&data_config_prfl))
                    .app_data(web::Data::clone(&data_profile_orm));
            }
        }
    }
}
