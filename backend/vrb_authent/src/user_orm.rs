use vrb_dbase::dbase::DbPool;

use crate::user_models::{CreateUser, ModifyUser, Profile, Session, User};

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

    /// Add a new entry (user).
    fn create_user(&self, create_user: CreateUser) -> Result<User, String>;

    /// Modify an entity (user).
    fn modify_user(&self, id: i32, modify_user: ModifyUser) -> Result<Option<User>, String>;

    /// Delete an entity (user).
    fn delete_user(&self, id: i32) -> Result<Option<User>, String>;

    /// Get an entity (profile) by USER_ID.
    fn get_profile_by_id(&self, user_id: i32) -> Result<Option<Profile>, String>;
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

    use crate::user_models::{CreateUser, ModifyUser, Profile, Session, User};
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

            let nickname2 = nickname.unwrap_or("").to_lowercase();
            let nickname2_len = nickname2.len();
            let email2 = email.unwrap_or("").to_lowercase();
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

        /// Add a new entry (user, profile).
        fn create_user(&self, create_user: CreateUser) -> Result<User, String> {
            let timer = if log_enabled!(Info) { Some(tm::now()) } else { None };

            let create_user2 = CreateUser::new(
                &create_user.nickname.to_lowercase(),
                &create_user.email.to_lowercase(),
                &create_user.password,
                create_user.role,
            );

            // Get a connection from the P2D2 pool.
            let mut conn = self.get_conn()?;

            // Run query using Diesel to add a new user entry.
            let user: User = diesel::insert_into(schema::users::table)
                .values(create_user2)
                .returning(User::as_returning())
                .get_result(&mut conn)
                .map_err(|e| format!("create_user: {}", e.to_string()))?;

            if let Some(timer) = timer {
                info!("create_user() time: {}", format!("{:.2?}", timer.elapsed()));
            }
            Ok(user)
        }

        /// Modify an entity (user).
        fn modify_user(&self, id: i32, modify_user: ModifyUser) -> Result<Option<User>, String> {
            let timer = if log_enabled!(Info) { Some(tm::now()) } else { None };

            let modify_user2 = ModifyUser::new(
                modify_user.nickname.map(|v| v.to_lowercase()),
                modify_user.email.map(|v| v.to_lowercase()),
                modify_user.password,
                modify_user.role,
            );

            // Get a connection from the P2D2 pool.
            let mut conn = self.get_conn()?;
            // Run query using Diesel to full or partially modify the user entry.
            let opt_user: Option<User> = diesel::update(schema::users::dsl::users.find(id))
                .set(&modify_user2)
                .returning(User::as_returning())
                .get_result(&mut conn)
                .optional()
                .map_err(|e| format!("modify_user: {}", e.to_string()))?;

            if let Some(timer) = timer {
                info!("modify_user() time: {}", format!("{:.2?}", timer.elapsed()));
            }
            Ok(opt_user)
        }

        /// Delete an entity (user).
        fn delete_user(&self, id: i32) -> Result<Option<User>, String> {
            let timer = if log_enabled!(Info) { Some(tm::now()) } else { None };

            // Get a connection from the P2D2 pool.
            let mut conn = self.get_conn()?;
            // Run query using Diesel to delete a entry (user).
            let result = diesel::delete(schema::users::dsl::users.find(id))
                .returning(User::as_returning())
                .get_result(&mut conn)
                .optional()
                .map_err(|e| format!("delete_user: {}", e.to_string()))?;

            if let Some(timer) = timer {
                info!("delete_user() time: {}", format!("{:.2?}", timer.elapsed()));
            }
            Ok(result)
        }

        /// Get an entity (profile) by USER_ID.
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
    }
}

#[cfg(any(test, feature = "mockdata"))]
pub mod tests {
    use actix_web::web;
    use chrono::Utc;
    use vrb_common::profile::{PROFILE_LOCALE_DEF, PROFILE_THEME_DARK, PROFILE_THEME_LIGHT_DEF};
    use vrb_dbase::enm_user_role::UserRole;

    use crate::{
        config_jwt,
        user_models::{CreateUser, ModifyUser, Profile, Session, User},
        user_orm::UserOrm,
    };

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

    pub const USER_IDS: [i32; 4] = [USER1_ID, USER2_ID, USER3_ID, USER4_ID];
    pub const USER_NAMES: [&str; 4] = [USER1_NAME, USER2_NAME, USER3_NAME, USER4_NAME];

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
                }
                None => None,
            };

            Ok(result)
        }

        /// Add a new entry (user, profile).
        fn create_user(&self, create_user: CreateUser) -> Result<User, String> {
            let nickname = create_user.nickname.to_lowercase();
            let email = create_user.email.to_lowercase();

            // Check the availability of the profile by nickname and email.
            let opt_user = self.find_user_by_nickname_or_email(Some(&nickname), Some(&email), false)?;
            if opt_user.is_some() {
                return Err("Profile already exists".to_string());
            }

            let idx: i32 = self.user_vec.len().try_into().unwrap();
            let user_id: i32 = USER1_ID + idx;
            let user = User::new(user_id, &nickname, &email, &create_user.password, create_user.role.unwrap_or(UserRole::User));
            Ok(user)
        }

        /// Modify an entity (user).
        fn modify_user(&self, id: i32, modify_user: ModifyUser) -> Result<Option<User>, String> {
            let opt_user1 = self.user_vec.iter().find(|user| (*user).id == id);
            let opt_user: Option<User> = if let Some(user1) = opt_user1 {
                let mut user = user1.clone();
                if let Some(nickname) = modify_user.nickname {
                    if nickname.len() > 0 {
                        user.nickname = nickname;
                    }
                }
                if let Some(email) = modify_user.email {
                    if email.len() > 0 {
                        user.email = email;
                    }
                }
                if let Some(password) = modify_user.password {
                    user.password = password;
                }
                if let Some(role) = modify_user.role {
                    user.role = role;
                }
                user.updated_at = Utc::now();
                Some(user)
            } else {
                None
            };
            Ok(opt_user)
        }

        /// Delete an entity (user).
        fn delete_user(&self, id: i32) -> Result<Option<User>, String> {
            let user_opt = self.user_vec.iter().find(|user| user.id == id);

            Ok(user_opt.map(|u| u.clone()))
        }

        /// Get an entity (profile) by USER_ID.
        fn get_profile_by_id(&self, user_id: i32) -> Result<Option<Profile>, String> {
            let opt_user: Option<User> = self.user_vec.iter().find(|user| user.id == user_id).map(|user| user.clone());

            let opt_profile = opt_user.map(|user| UserOrmTest::profile(user.id));

            Ok(opt_profile)
        }
    }

    pub struct UserOrmTest {}

    impl UserOrmTest {
        pub fn users(roles: &[u8]) -> (Vec<User>, Vec<Session>) {
            let mut user_vec: Vec<User> = Vec::new();
            let mut session_vec: Vec<Session> = Vec::new();
            let user_ids = USER_IDS.clone();

            let len = if roles.len() > user_ids.len() { user_ids.len() } else { roles.len() };
            for index in 0..len {
                let user_id = user_ids.get(index).unwrap().clone();
                let nickname = USER_NAMES.get(index).unwrap().to_lowercase();
                #[rustfmt::skip]
                let role = if *(roles.get(index).unwrap()) == ADMIN { UserRole::Admin } else { UserRole::User };

                let user = User::new(user_id, &nickname, &format!("{}@gmail.com", nickname), "", role);
                user_vec.push(user);
                let num_token = if user_id == USER1_ID {
                    Some(config_jwt::tests::get_num_token(user_id))
                } else {
                    None
                };
                session_vec.push(Session { user_id, num_token });
            }
            (user_vec, session_vec)
        }
        pub fn cfg_user_orm(data_p: (Vec<User>, Vec<Session>)) -> impl FnOnce(&mut web::ServiceConfig) {
            move |config: &mut web::ServiceConfig| {
                let mut user_orm_app = UserOrmApp::new();
                user_orm_app.user_vec.extend(data_p.0);
                user_orm_app.session_vec.extend(data_p.1);

                let data_user_orm = web::Data::new(user_orm_app);
                config.app_data(web::Data::clone(&data_user_orm));
            }
        }
        pub fn profile(user_id: i32) -> Profile {
            let descript = format!("descript_{}", user_id);
            #[rustfmt::skip]
            let theme = if user_id % 2 == 0 { PROFILE_THEME_DARK.to_owned() } else { PROFILE_THEME_LIGHT_DEF.to_owned() };
            #[rustfmt::skip]
            let locale = if user_id % 2 == 0 { PROFILE_LOCALE_DEF.to_owned() } else { "en-US".to_owned() };
            Profile::new(user_id, None, Some(descript), Some(theme), Some(locale))
        }
    }
}
