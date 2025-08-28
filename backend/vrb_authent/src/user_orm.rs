use vrb_dbase::dbase::DbPool;

use crate::profile_models2::{CreateProfile, Profile, ModifyProfile};
use crate::user_models::{Session, User};

pub trait UserOrm {
    /// Get an entity (user) by ID.
    fn get_user_by_id(&self, id: i32, is_password: bool) -> Result<Option<User>, String>;

    /// Get an entity (session) by ID.
    fn get_session_by_id(&self, user_id: i32) -> Result<Option<Session>, String>;

    /// Modify the entity (session).
    fn modify_session(&self, user_id: i32, num_token: Option<i32>) -> Result<Option<Session>, String>;
    // There is no need to delete the entity (session), since it is deleted cascade when deleting an entry in the users table.

    /// Find for an entity (user) by nickname or email.
    #[rustfmt::skip]
    fn find_user_by_nickname_or_email(
        &self, nickname: Option<&str>, email: Option<&str>, is_password: bool,
    ) -> Result<Option<User>, String>;

    /// Add a new entry (user, profile).
    fn create_profile_user(&self, create_profile: CreateProfile) -> Result<Profile, String>;

    /// Modify an entity (user, profile).
    fn modify_profile(&self, user_id: i32, modify_profile: ModifyProfile) -> Result<Option<Profile>, String>;

}

#[cfg(not(all(test, feature = "mockdata")))]
pub fn get_user_orm_app(pool: DbPool) -> impls::UserOrmApp {
    impls::UserOrmApp::new(pool)
}
#[cfg(all(test, feature = "mockdata"))]
pub fn get_user_orm_app(_: DbPool) -> tests::UserOrmApp {
    tests::UserOrmApp::new()
}

#[cfg(not(all(test, feature = "mockdata")))]
pub mod impls {
    use std::time::Instant as tm;

    use diesel::{self, prelude::*, sql_types};
    use log::{info, log_enabled, Level::Info};
    use vrb_dbase::{dbase, schema};

    use crate::profile_models2::{CreateProfile, Profile, ModifyProfile};
    use crate::user_models::{Session, User};
    use crate::user_orm::UserOrm;

    pub const CONN_POOL: &str = "ConnectionPool";

    #[derive(Debug, Clone)]
    pub struct UserOrmApp {
        pub pool: dbase::DbPool,
    }

    impl UserOrmApp {
        pub fn new(pool: dbase::DbPool) -> Self {
            UserOrmApp { pool }
        }
        pub fn get_conn(&self) -> Result<dbase::DbPooledConnection, String> {
            (&self.pool).get().map_err(|e| format!("{}: {}", CONN_POOL, e.to_string()))
        }
    }

    impl UserOrm for UserOrmApp {
        /// Get an entity (user) by ID.
        fn get_user_by_id(&self, id: i32, is_password: bool) -> Result<Option<User>, String> {
            let timer = if log_enabled!(Info) { Some(tm::now()) } else { None };
            // Get a connection from the P2D2 pool.
            let mut conn = self.get_conn()?;

            #[rustfmt::skip]
            let query =
                diesel::sql_query("select * from find_user($1, NULL, NULL, $2);")
                .bind::<sql_types::Integer, _>(id)
                .bind::<sql_types::Bool, _>(is_password);

            let opt_user = query
                .get_result::<User>(&mut conn)
                .optional()
                .map_err(|e| format!("find_user_by_id: {}", e.to_string()))?;

            if let Some(timer) = timer {
                info!("get_user_by_id() time: {}", format!("{:.2?}", timer.elapsed()));
            }
            Ok(opt_user)
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

        /// Find for an entity (user) by nickname or email.
        fn find_user_by_nickname_or_email(
            &self,
            nickname: Option<&str>,
            email: Option<&str>,
            is_password: bool,
        ) -> Result<Option<User>, String> {
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

            let query = diesel::sql_query("select * from find_user(NULL, $1, $2, $3);")
                .bind::<sql_types::Text, _>(nickname2)
                .bind::<sql_types::Text, _>(email2)
                .bind::<sql_types::Bool, _>(is_password);

            // Run query using Diesel to find user by id (and user_id) and return it.
            let opt_user = query
                .get_result::<User>(&mut conn)
                .optional()
                .map_err(|e| format!("find_user: {}", e.to_string()))?;

            if let Some(timer) = timer {
                #[rustfmt::skip]
                info!("find_user_by_nickname_or_email() time: {}", format!("{:.2?}", timer.elapsed()));
            }
            Ok(opt_user)
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

        /// Modify an entity (user, profile).
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
    }
}

#[cfg(any(test, feature = "mockdata"))]
pub mod tests {
    use actix_web::web;
    use chrono::Utc;
    use vrb_dbase::enm_user_role::UserRole;
    use vrb_tools::token_coding;

    use crate::config_jwt;
    use crate::profile_models2::{CreateProfile, Profile, ModifyProfile};
    use crate::user_models::{Session, User};
    use crate::user_orm::UserOrm;

    pub const ADMIN: u8 = 0;
    pub const USER: u8 = 1;

    pub const USER1: usize = 0;
    pub const USER2: usize = 1;
    pub const USER3: usize = 2;
    pub const USER4: usize = 3;

    pub const USER1_ID: i32 = 1100;
    pub const USER2_ID: i32 = 1101;
    pub const USER3_ID: i32 = 1102;
    pub const USER4_ID: i32 = 1103;

    pub const USER1_NAME: &str = "oliver_taylor";
    pub const USER2_NAME: &str = "robert_brown";
    pub const USER3_NAME: &str = "mary_williams";
    pub const USER4_NAME: &str = "ava_wilson";

    pub const USER1_NUM_TOKEN: i32 = 20000 + USER1_ID; //  1234;

    #[derive(Debug, Clone)]
    pub struct UserOrmApp {
        pub user_vec: Vec<User>,
        pub session_vec: Vec<Session>,
    }

    impl UserOrmApp {
        /// Create a new instance.
        pub fn new() -> Self {
            UserOrmApp {
                user_vec: Vec::new(),
                session_vec: Vec::new(),
            }
        }
        /// Create a new instance with the specified user list.
        /// Sessions are taken from "sessions", if it is empty, they are created automatically.
        pub fn create(users: &[User], sessions: &[Session]) -> Self {
            let mut user_vec: Vec<User> = Vec::new();
            let mut session_vec: Vec<Session> = Vec::new();
            let mut sessions2: Vec<Session> = sessions.to_vec();
            for (idx, user) in users.iter().enumerate() {
                let delta: i32 = idx.try_into().unwrap();
                let user_id = USER1_ID + delta;
                let mut user2 = Self::new_user(user_id, &user.nickname, &user.email, &user.password, user.role);
                user2.created_at = user.created_at;
                user2.updated_at = user.updated_at;
                user_vec.push(user2);

                if sessions2.len() > 0 {
                    let opt_session = sessions2.iter_mut().find(|v| (*v).user_id == user.id);
                    if let Some(session) = opt_session {
                        session_vec.push(Session::new(user_id, session.num_token));
                        session.user_id = 0;
                    }
                } else {
                    session_vec.push(Session::new(user_id, None));
                }
            }
            for session in sessions2.iter() {
                if USER1_ID <= session.user_id {
                    session_vec.push(Session::new(session.user_id, session.num_token));
                }
            }

            UserOrmApp {
                user_vec: user_vec,
                session_vec,
            }
        }
        /// Create a new instance of the User entity.
        pub fn new_user(id: i32, nickname: &str, email: &str, password: &str, role: UserRole) -> User {
            User::new(id, &nickname.to_lowercase(), &email.to_lowercase(), password, role.clone())
        }
        /// Create a new instance of the Session entity.
        pub fn new_session(user_id: i32, num_token: Option<i32>) -> Session {
            Session { user_id, num_token }
        }
    }

    impl UserOrm for UserOrmApp {
        /// Get an entity (user) by ID.
        fn get_user_by_id(&self, id: i32, is_password: bool) -> Result<Option<User>, String> {
            let opt_user = self.user_vec.iter().find(|user| user.id == id).map(|user| user.clone());

            let result = match opt_user {
                Some(mut user) if !is_password => {
                    user.password = "".to_string();
                    Some(user)
                }
                Some(v) => Some(v),
                None => None,
            };

            Ok(result)
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

        /// Find for an entity (user) by nickname or email.
        fn find_user_by_nickname_or_email(
            &self,
            nickname: Option<&str>,
            email: Option<&str>,
            is_password: bool,
        ) -> Result<Option<User>, String> {
            let nickname2 = nickname.unwrap_or(&"".to_string()).to_lowercase();
            let nickname2_len = nickname2.len();
            let email2 = email.unwrap_or(&"".to_string()).to_lowercase();
            let email2_len = email2.len();

            if nickname2_len == 0 && email2_len == 0 {
                return Ok(None);
            }

            let opt_user = self
                .user_vec
                .iter()
                .find(|u| (nickname2_len > 0 && u.nickname == nickname2) || (email2_len > 0 && u.email == email2))
                .map(|u| u.clone());

            let result = match opt_user {
                Some(mut user) => {
                    if !is_password {
                        user.password = "".to_string();
                    }
                    Some(user)
                },
                None => None,
            };

            Ok(result)
        }

        /// Add a new entry (user, profile).
        fn create_profile_user(&self, create_profile: CreateProfile) -> Result<Profile, String> {
            let nickname = create_profile.nickname.to_lowercase();
            let email = create_profile.email.to_lowercase();

            // Check the availability of the profile by nickname and email.
            let opt_profile = self.find_user_by_nickname_or_email(Some(&nickname), Some(&email), false)?;
            if opt_profile.is_some() {
                return Err("Profile already exists".to_string());
            }

            let idx: i32 = self.user_vec.len().try_into().unwrap();
            let user_id: i32 = USER1_ID + idx;

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
            let opt_user = self.user_vec.iter().find(|user| (*user).id == user_id);
            let opt_profile3: Option<Profile> = if let Some(user) = opt_user {
                let profile2 = Profile {
                    user_id: user.id,
                    nickname: modify_profile.nickname.unwrap_or(user.nickname.clone()),
                    email: modify_profile.email.unwrap_or(user.email.clone()),
                    password: modify_profile.password.unwrap_or(user.password.clone()),
                    role: modify_profile.role.unwrap_or(user.role.clone()),
                    avatar: modify_profile.avatar.unwrap_or(None), // # user.avatar.clone()),
                    descript: modify_profile.descript.or(None), // # user.descript.clone()),
                    theme: modify_profile.theme.or(None), // # user.theme.clone()),
                    locale: modify_profile.locale.or(None), // # user.locale.clone()),
                    created_at: user.created_at,
                    updated_at: Utc::now(),
                };
                Some(profile2)
            } else {
                None
            };
            Ok(opt_profile3)
        }

    }

    pub struct UserOrmTest {}

    impl UserOrmTest {
        pub fn user_ids() -> Vec<i32> {
            vec![USER1_ID, USER2_ID, USER3_ID, USER4_ID]
        }
        pub fn get_user_name(user_id: i32) -> String {
            match user_id {
                USER1_ID => USER1_NAME,
                USER2_ID => USER2_NAME,
                USER3_ID => USER3_NAME,
                USER4_ID => USER4_NAME,
                _ => "",
            }
            .to_string()
        }
        pub fn get_num_token(user_id: i32) -> i32 {
            40000 + user_id
        }
        pub fn get_token(user_id: i32) -> String {
            let config_jwt = config_jwt::get_test_config();
            let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();
            let num_token = Self::get_num_token(user_id);
            token_coding::encode_token(user_id, num_token, &jwt_secret, config_jwt.jwt_access).unwrap()
        }
        pub fn users(roles: &[u8]) -> (Vec<User>, Vec<Session>) {
            let mut user_vec: Vec<User> = Vec::new();
            let mut session_vec: Vec<Session> = Vec::new();
            let user_ids = UserOrmTest::user_ids();
            let len = if roles.len() > user_ids.len() { user_ids.len() } else { roles.len() };
            for index in 0..len {
                let user_id = user_ids.get(index).unwrap().clone();
                let nickname = Self::get_user_name(user_id).clone().to_lowercase();
                let email = format!("{}@gmail.com", nickname);
                #[rustfmt::skip]
                let role = if roles.get(index).unwrap().clone() == ADMIN { UserRole::Admin } else { UserRole::User };

                let user = User::new(user_id, &nickname, &email, "", role);
                user_vec.push(user);
                let num_token = if user_id == USER1_ID { Some(Self::get_num_token(user_id)) } else { None };
                session_vec.push(Session { user_id, num_token });
            }
            let user_orm_app = UserOrmApp { user_vec, session_vec };

            (user_orm_app.user_vec, user_orm_app.session_vec)
        }
        pub fn cfg_config_jwt(config_jwt: config_jwt::ConfigJwt) -> impl FnOnce(&mut web::ServiceConfig) {
            move |config: &mut web::ServiceConfig| {
                let data_config_jwt = web::Data::new(config_jwt);
                config.app_data(web::Data::clone(&data_config_jwt));
            }
        }
        pub fn cfg_user_orm(data_p: (Vec<User>, Vec<Session>)) -> impl FnOnce(&mut web::ServiceConfig) {
            move |config: &mut web::ServiceConfig| {
                let data_user_orm = web::Data::new(UserOrmApp::create(&data_p.0, &data_p.1));

                config.app_data(web::Data::clone(&data_user_orm));
            }
        }
    }
}
