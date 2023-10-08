use crate::sessions::session_models;

pub trait SessionOrm {
    /// Search for a user by ID and return it.
    fn find_by_id(&self, user_id: i32) -> Result<Option<session_models::Session>, String>;
    /// Add a new session entry.
    fn create_session(
        &self,
        session: &session_models::Session,
    ) -> Result<session_models::Session, String>;
    /// Perform a full or partial change to a session record.
    fn modify_session(
        &self,
        user_id: i32,
        num_token: Option<i32>,
    ) -> Result<Option<session_models::Session>, String>;
}

#[cfg(not(feature = "mockdata"))]
use diesel::prelude::*;

#[cfg(not(feature = "mockdata"))]
use crate::dbase;
#[cfg(not(feature = "mockdata"))]
use crate::schema;

#[cfg(not(feature = "mockdata"))]
#[derive(Debug, Clone)]
pub struct SessionOrmApp {
    pub pool: dbase::DbPool,
}

#[cfg(not(feature = "mockdata"))]
impl SessionOrmApp {
    pub fn new(pool: dbase::DbPool) -> Self {
        SessionOrmApp { pool }
    }
    pub fn get_conn(&self) -> Result<dbase::DbPooledConnection, String> {
        (&self.pool).get().map_err(|e| e.to_string())
    }
}

#[cfg(not(feature = "mockdata"))]
impl SessionOrm for SessionOrmApp {
    /// Search for a user by ID and return it.
    fn find_by_id(&self, user_id: i32) -> Result<Option<session_models::Session>, String> {
        // Get a connection from the P2D2 pool.
        let mut conn = self.get_conn()?;
        // Run query using Diesel to find user by id and return it.
        let session_opt: Option<session_models::Session> = schema::sessions::table
            .filter(schema::sessions::dsl::user_id.eq(user_id))
            .first::<session_models::Session>(&mut conn)
            .optional()
            .map_err(|e| {
                log::debug!("SessionOrmError: {}", e.to_string());
                e.to_string()
            })?;

        Ok(session_opt)
    }
    /// Add a new session entry.
    fn create_session(
        &self,
        session: &session_models::Session,
    ) -> Result<session_models::Session, String> {
        // Get a connection from the P2D2 pool.
        let mut conn = self.get_conn()?;
        // Run query using Diesel to add a new session entry.
        let session: session_models::Session = diesel::insert_into(schema::sessions::table)
            .values(session)
            .returning(session_models::Session::as_returning())
            .get_result(&mut conn)
            .map_err(|e| {
                log::debug!("SessionOrmError: {}", e.to_string());
                e.to_string()
            })?;

        Ok(session)
    }
    /// Perform a full or partial change to a session record.
    fn modify_session(
        &self,
        user_id: i32,
        num_token: Option<i32>,
    ) -> Result<Option<session_models::Session>, String> {
        // Get a connection from the P2D2 pool.
        let mut conn = self.get_conn()?;
        // Run query using Diesel to full or partially modify the session entry.
        let result = diesel::update(schema::sessions::dsl::sessions.find(user_id))
            .set(schema::sessions::dsl::num_token.eq(num_token))
            .returning(session_models::Session::as_returning())
            .get_result(&mut conn)
            .optional()
            .map_err(|e| {
                log::debug!("SessionOrmError: {}", e.to_string());
                e.to_string()
            })?;

        Ok(result)
    }
}

#[cfg(feature = "mockdata")]
pub mod tests {

    use super::session_models;
    use super::*;

    #[derive(Debug, Clone)]
    pub struct SessionOrmApp {
        sessions: Vec<session_models::Session>,
    }

    impl SessionOrmApp {
        /// Create a new instance.
        pub fn new() -> Self {
            SessionOrmApp {
                sessions: Vec::new(),
            }
        }
        /// Create a new instance with the specified sessions.
        #[cfg(test)]
        pub fn create(session_list: Vec<session_models::Session>) -> Self {
            let mut sessions: Vec<session_models::Session> = Vec::new();
            for session in session_list.iter() {
                sessions.push(session_models::Session {
                    user_id: session.user_id,
                    num_token: session.num_token,
                });
            }
            SessionOrmApp { sessions }
        }
        /// Create a new entity instance.
        pub fn new_session(user_id: i32, num_token: i32) -> session_models::Session {
            session_models::Session {
                user_id: user_id,
                num_token: Some(num_token),
            }
        }
    }

    impl SessionOrm for SessionOrmApp {
        /// Search for a user by ID and return it.
        fn find_by_id(&self, user_id: i32) -> Result<Option<&session_models::Session>, String> {
            let session_opt: Option<&session_models::Session> = self.find(user_id);
            Ok(session_opt)
        }
        /// Add a new session entry.
        fn create_session(
            &self,
            session: &session_models::Session,
        ) -> Result<session_models::Session, String> {
            let id = session.user_id;
            // Check the availability of the user ID.
            let user_opt = self.find_by_user_id(id)?;
            if user_opt.is_some() {
                return Err("Session already exists".to_string());
            }
            let session_saved = SessionOrmApp::new_session(session.user_id, session.num_token);

            Ok(session_saved)
        }
        /// Perform a full or partial change to a session record.
        fn modify_session(
            &self,
            user_id: i32,
            num_token: Option<i32>,
        ) -> Result<Option<session_models::Session>, String> {
            let session_opt: Option<&session_models::Session> = self.find(user_id);

            if session_opt.is_none() {
                return Ok(None);
            }
            let mut res_session = session_opt.unwrap().clone();
            res_session.num_token = num_token;

            Ok(Some(res_session))
        }
    }
}
