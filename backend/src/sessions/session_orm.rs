use crate::sessions::session_models::Session;

pub trait SessionOrm {
    /// Find for an entity (session) by id.
    fn find_session_by_id(&self, user_id: i32) -> Result<Option<Session>, String>;
    /// Add a new entity (session).
    fn create_session(&self, session: &Session) -> Result<Session, String>;
    /// Modify the entity (session).
    fn modify_session(
        &self,
        user_id: i32,
        num_token: Option<i32>,
    ) -> Result<Option<Session>, String>;
    // There is no need to delete the entity (session), since it is deleted cascade when deleting an entry in the users table.
}

pub mod cfg {
    use crate::dbase::DbPool;

    #[cfg(not(feature = "mockdata"))]
    use super::inst::SessionOrmApp;
    #[cfg(not(feature = "mockdata"))]
    pub fn get_session_orm_app(pool: DbPool) -> SessionOrmApp {
        SessionOrmApp::new(pool)
    }

    #[cfg(feature = "mockdata")]
    use super::tests::SessionOrmApp;
    #[cfg(feature = "mockdata")]
    pub fn get_session_orm_app(_: DbPool) -> SessionOrmApp {
        SessionOrmApp::new()
    }
}

#[cfg(not(feature = "mockdata"))]
pub mod inst {

    use diesel::{self, prelude::*};

    use crate::dbase;
    use crate::schema;

    use super::*;

    pub const CONN_POOL: &str = "ConnectionPool";
    pub const DB_SESSION: &str = "Db_Session";

    #[derive(Debug, Clone)]
    pub struct SessionOrmApp {
        pub pool: dbase::DbPool,
    }

    impl SessionOrmApp {
        pub fn new(pool: dbase::DbPool) -> Self {
            SessionOrmApp { pool }
        }
        pub fn get_conn(&self) -> Result<dbase::DbPooledConnection, String> {
            (&self.pool).get().map_err(|e| format!("{CONN_POOL}: {}", e.to_string()))
        }
    }

    impl SessionOrm for SessionOrmApp {
        /// Find for an entity (session) by id.
        fn find_session_by_id(&self, user_id: i32) -> Result<Option<Session>, String> {
            // Get a connection from the P2D2 pool.
            let mut conn = self.get_conn()?;
            // Run query using Diesel to find user by id and return it.
            let session_opt: Option<Session> = schema::sessions::table
                .filter(schema::sessions::dsl::user_id.eq(user_id))
                .first::<Session>(&mut conn)
                .optional()
                .map_err(|e| format!("{}: {}", DB_SESSION, e.to_string()))?;

            Ok(session_opt)
        }
        /// Add a new session entry.
        fn create_session(&self, session: &Session) -> Result<Session, String> {
            // Get a connection from the P2D2 pool.
            let mut conn = self.get_conn()?;
            // Run query using Diesel to add a new session entry.
            let session: Session = diesel::insert_into(schema::sessions::table)
                .values(session)
                .returning(Session::as_returning())
                .get_result(&mut conn)
                .map_err(|e| format!("{}: {}", DB_SESSION, e.to_string()))?;

            Ok(session)
        }
        /// Perform a full or partial change to a session record.
        fn modify_session(
            &self,
            user_id: i32,
            num_token: Option<i32>,
        ) -> Result<Option<Session>, String> {
            // Get a connection from the P2D2 pool.
            let mut conn = self.get_conn()?;
            // Run query using Diesel to full or partially modify the session entry.
            let result = diesel::update(schema::sessions::dsl::sessions.find(user_id))
                .set(schema::sessions::dsl::num_token.eq(num_token))
                .returning(Session::as_returning())
                .get_result(&mut conn)
                .optional()
                .map_err(|e| format!("{}: {}", DB_SESSION, e.to_string()))?;

            Ok(result)
        }
    }
}

#[cfg(feature = "mockdata")]
pub mod tests {

    use super::*;

    #[derive(Debug, Clone)]
    pub struct SessionOrmApp {
        session_vec: Vec<Session>,
    }

    impl SessionOrmApp {
        /// Create a new instance.
        pub fn new() -> Self {
            SessionOrmApp {
                session_vec: Vec::new(),
            }
        }
        /// Create a new instance with the specified session list.
        #[cfg(test)]
        pub fn create(session_list: Vec<Session>) -> Self {
            let mut session_vec: Vec<Session> = Vec::new();
            for session in session_list.iter() {
                session_vec.push(Self::new_session(session.user_id, session.num_token));
            }

            SessionOrmApp { session_vec }
        }
        /// Create a new entity instance.
        pub fn new_session(user_id: i32, num_token: Option<i32>) -> Session {
            Session { user_id, num_token }
        }
    }

    impl SessionOrm for SessionOrmApp {
        /// Find for an entity (session) by id.
        fn find_session_by_id(&self, user_id: i32) -> Result<Option<Session>, String> {
            let session_opt: Option<Session> = self
                .session_vec
                .iter()
                .find(|session| session.user_id == user_id)
                .map(|session| session.clone());

            Ok(session_opt)
        }
        /// Add a new entity (session).
        fn create_session(&self, session: &Session) -> Result<Session, String> {
            let id = session.user_id;
            // Check the availability of the user ID.
            let user_opt = self.find_session_by_id(id)?;
            if user_opt.is_some() {
                return Err("Session already exists".to_string());
            }
            let session_saved = Self::new_session(session.user_id, session.num_token);

            Ok(session_saved)
        }
        /// Modify the entity (session).
        fn modify_session(
            &self,
            user_id: i32,
            num_token: Option<i32>,
        ) -> Result<Option<Session>, String> {
            let session_opt: Option<Session> = self.find_session_by_id(user_id)?;
            let mut res_session = match session_opt {
                Some(v) => v,
                None => {
                    return Ok(None);
                }
            };
            let new_session = Self::new_session(user_id, num_token);
            res_session.num_token = new_session.num_token;

            Ok(Some(res_session))
        }
    }
}
