use crate::user_auth_models::{Session, User};

pub trait UserAuthOrm {
    /// Get an entity (user) by ID.
    fn get_user_by_id(&self, id: i32, is_password: bool) -> Result<Option<User>, String>;

    /// Get an entity (session) by ID.
    fn get_session_by_id(&self, user_id: i32) -> Result<Option<Session>, String>;
}

pub mod impls {
    use std::time::Instant as tm;

    use diesel::{self, prelude::*, sql_types};
    use log::{info, log_enabled, Level::Info};

    use crate::user_auth_models::{Session, User};
    use crate::user_auth_orm::UserAuthOrm;
    use crate::{dbase, schema};

    pub const CONN_POOL: &str = "ConnectionPool";

    // pub fn get_user_auth_orm_app(pool: DbPool) -> UserAuthApp {
    //     UserAuthApp::new(pool)
    // }

    #[derive(Debug, Clone)]
    pub struct UserAuthOrmApp {
        pub pool: dbase::DbPool,
    }

    impl UserAuthOrmApp {
        pub fn new(pool: dbase::DbPool) -> Self {
            UserAuthOrmApp { pool }
        }
        pub fn get_conn(&self) -> Result<dbase::DbPooledConnection, String> {
            (&self.pool).get().map_err(|e| format!("{}: {}", CONN_POOL, e.to_string()))
        }
    }

    impl UserAuthOrm for UserAuthOrmApp {
        /// Get an entity (user) by ID.
        fn get_user_by_id(&self, id: i32, is_password: bool) -> Result<Option<User>, String> {
            let timer = if log_enabled!(Info) { Some(tm::now()) } else { None };
            // Get a connection from the P2D2 pool.
            let mut conn = self.get_conn()?;

            #[rustfmt::skip]
            let query =
                diesel::sql_query("select * from find_user_by_id($1, $2);")
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
    }
}

pub mod tests {
    use actix_web::web;
    use vrb_tools::token_coding;

    use crate::config_jwt;
    use crate::db_enums::UserRole;
    use crate::user_auth_models::{Session, User};
    use crate::user_auth_orm::UserAuthOrm;

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

    // pub fn get_user_auth_orm_app(_: DbPool) -> UserAuthApp {
    //     UserAuthApp::new()
    // }

    #[derive(Debug, Clone)]
    pub struct UserAuthOrmApp {
        pub user_vec: Vec<User>,
        pub session_vec: Vec<Session>,
    }

    impl UserAuthOrmApp {
        /// Create a new instance.
        pub fn new() -> Self {
            UserAuthOrmApp {
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

            UserAuthOrmApp {
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

    impl UserAuthOrm for UserAuthOrmApp {
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
    }

    pub struct UserAuthOrmTest {}

    impl UserAuthOrmTest {
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
            let user_ids = UserAuthOrmTest::user_ids();
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
            let user_auth_orm_app = UserAuthOrmApp { user_vec, session_vec };

            (user_auth_orm_app.user_vec, user_auth_orm_app.session_vec)
        }
        pub fn cfg_config_jwt(config_jwt: config_jwt::ConfigJwt) -> impl FnOnce(&mut web::ServiceConfig) {
            move |config: &mut web::ServiceConfig| {
                let data_config_jwt = web::Data::new(config_jwt);
                config.app_data(web::Data::clone(&data_config_jwt));
            }
        }
        pub fn cfg_user_auth_orm(data_p: (Vec<User>, Vec<Session>)) -> impl FnOnce(&mut web::ServiceConfig) {
            move |config: &mut web::ServiceConfig| {
                let data_user_auth_orm = web::Data::new(UserAuthOrmApp::create(&data_p.0, &data_p.1));

                config.app_data(web::Data::clone(&data_user_auth_orm));
            }
        }
    }
}
