use super::user_models::{CreateUserRecovery, UserRecovery};

pub const DURATION_IN_DAYS: u16 = 90;

pub trait UserRecoveryOrm {
    /// Get an entity (user_recovery) by ID.
    fn get_user_recovery_by_id(&self, id: i32) -> Result<Option<UserRecovery>, String>;
    /// Find for an entity (user_recovery) by user_id.
    fn find_user_recovery_by_user_id(&self, user_id: i32) -> Result<Option<UserRecovery>, String>;
    /// Add a new entity (user_recovery).
    fn create_user_recovery(&self, create_user_recovery: CreateUserRecovery) -> Result<UserRecovery, String>;
    /// Modify an entity (user_recovery).
    fn modify_user_recovery(
        &self,
        id: i32,
        modify_user_recovery: CreateUserRecovery,
    ) -> Result<Option<UserRecovery>, String>;
    /// Delete an entity (user_recovery).
    fn delete_user_recovery(&self, id: i32) -> Result<usize, String>;
    /// Delete all entities (user_recovery) with an inactive "final_date".
    fn delete_inactive_final_date(&self, duration_in_days: Option<u16>) -> Result<usize, String>;
}

pub mod cfg {
    use crate::dbase::DbPool;

    #[cfg(not(feature = "mockdata"))]
    use super::impls::UserRecoveryOrmApp;
    #[cfg(not(feature = "mockdata"))]
    pub fn get_user_recovery_orm_app(pool: DbPool) -> UserRecoveryOrmApp {
        UserRecoveryOrmApp::new(pool)
    }

    #[cfg(feature = "mockdata")]
    use super::tests::UserRecoveryOrmApp;
    #[cfg(feature = "mockdata")]
    pub fn get_user_recovery_orm_app(_: DbPool) -> UserRecoveryOrmApp {
        UserRecoveryOrmApp::new()
    }
}

#[cfg(not(feature = "mockdata"))]
pub mod impls {
    use std::time::Instant as tm;

    use chrono::{Duration, Utc};
    use diesel::{self, prelude::*};
    use log::{info, log_enabled, Level::Info};
    use schema::user_recovery::dsl;

    use crate::dbase;
    use crate::schema;
    use crate::users::{user_models::UserRecovery, user_recovery_orm::DURATION_IN_DAYS};

    use super::*;

    pub const CONN_POOL: &str = "ConnectionPool";

    #[derive(Debug, Clone)]
    pub struct UserRecoveryOrmApp {
        pub pool: dbase::DbPool,
    }

    impl UserRecoveryOrmApp {
        pub fn new(pool: dbase::DbPool) -> Self {
            UserRecoveryOrmApp { pool }
        }
        pub fn get_conn(&self) -> Result<dbase::DbPooledConnection, String> {
            (&self.pool).get().map_err(|e| format!("{CONN_POOL}: {}", e.to_string()))
        }
    }

    impl UserRecoveryOrm for UserRecoveryOrmApp {
        /// Get an entity (user_recovery) by ID.
        fn get_user_recovery_by_id(&self, id: i32) -> Result<Option<UserRecovery>, String> {
            let timer = if log_enabled!(Info) { Some(tm::now()) } else { None };
            // Get a connection from the P2D2 pool.
            let mut conn = self.get_conn()?;
            // Run query using Diesel to find user by id and return it.
            let result = schema::user_recovery::table
                .filter(dsl::id.eq(id))
                .first::<UserRecovery>(&mut conn)
                .optional()
                .map_err(|e| format!("find_user_recovery_by_id: {}", e.to_string()))?;

            if let Some(timer) = timer {
                info!("get_user_recovery_by_id() time: {}", format!("{:.2?}", timer.elapsed()));
            }
            Ok(result)
        }

        /// Find for an entity (user_recovery) by user_id.
        fn find_user_recovery_by_user_id(&self, user_id: i32) -> Result<Option<UserRecovery>, String> {
            let timer = if log_enabled!(Info) { Some(tm::now()) } else { None };
            // Get a connection from the P2D2 pool.
            let mut conn = self.get_conn()?;
            let now = Utc::now();
            // Run query using Diesel to find user by user_id and return it (where final_date > now).
            let result = schema::user_recovery::table
                .filter(dsl::user_id.eq(user_id).and(dsl::final_date.gt(now)))
                .first::<UserRecovery>(&mut conn)
                .optional()
                .map_err(|e| format!("find_user_recovery_by_user_id: {}", e.to_string()))?;

            if let Some(timer) = timer {
                #[rustfmt::skip]
                info!("find_user_recovery_by_user_id() time: {}", format!("{:.2?}", timer.elapsed()));
            }
            Ok(result)
        }

        /// Add a new entity (user_recovery).
        fn create_user_recovery(&self, create_user_recovery: CreateUserRecovery) -> Result<UserRecovery, String> {
            let timer = if log_enabled!(Info) { Some(tm::now()) } else { None };
            // Get a connection from the P2D2 pool.
            let mut conn = self.get_conn()?;
            // Run query using Diesel to add a new user entry.
            let user_recovery: UserRecovery = diesel::insert_into(schema::user_recovery::table)
                .values(create_user_recovery)
                .returning(UserRecovery::as_returning())
                .get_result(&mut conn)
                .map_err(|e| format!("create_user_recovery: {}", e.to_string()))?;

            if let Some(timer) = timer {
                info!("create_user_recovery() time: {}", format!("{:.2?}", timer.elapsed()));
            }
            Ok(user_recovery)
        }

        /// Modify an entity (user_recovery).
        fn modify_user_recovery(
            &self,
            id: i32,
            create_user_recovery: CreateUserRecovery,
        ) -> Result<Option<UserRecovery>, String> {
            let timer = if log_enabled!(Info) { Some(tm::now()) } else { None };
            // Get a connection from the P2D2 pool.
            let mut conn = self.get_conn()?;
            // Run query using Diesel to full or partially modify the user entry.
            let result = diesel::update(dsl::user_recovery.find(id))
                .set(&create_user_recovery)
                .returning(UserRecovery::as_returning())
                .get_result(&mut conn)
                .optional()
                .map_err(|e| format!("modify_user_recovery: {}", e.to_string()))?;

            if let Some(timer) = timer {
                info!("modify_user_recovery() time: {}", format!("{:.2?}", timer.elapsed()));
            }
            Ok(result)
        }

        /// Delete an entity (user_recovery).
        fn delete_user_recovery(&self, id: i32) -> Result<usize, String> {
            let timer = if log_enabled!(Info) { Some(tm::now()) } else { None };
            // Get a connection from the P2D2 pool.
            let mut conn = self.get_conn()?;
            // Run query using Diesel to delete a entry (user_recovery).
            let count: usize = diesel::delete(dsl::user_recovery.find(id))
                .execute(&mut conn)
                .map_err(|e| format!("delete_user_recovery: {}", e.to_string()))?;

            if let Some(timer) = timer {
                info!("delete_user_recovery() time: {}", format!("{:.2?}", timer.elapsed()));
            }
            Ok(count)
        }

        /// Delete all entities (user_recovery) with an inactive "final_date".
        fn delete_inactive_final_date(&self, duration_in_days: Option<u16>) -> Result<usize, String> {
            let timer = if log_enabled!(Info) { Some(tm::now()) } else { None };
            let now = Utc::now();
            let duration = duration_in_days.unwrap_or(DURATION_IN_DAYS.into());
            let start_day_time = now - Duration::days(duration.into());
            let end_day_time = now.clone();

            // Get a connection from the P2D2 pool.
            let mut conn = self.get_conn()?;
            // Run query using Diesel to delete a entry (user_recovery).
            let count: usize = diesel::delete(
                schema::user_recovery::table
                    .filter(dsl::final_date.gt(start_day_time).and(dsl::final_date.lt(end_day_time))),
            )
            .execute(&mut conn)
            .map_err(|e| format!("delete_inactive_final_date: {}", e.to_string()))?;

            if let Some(timer) = timer {
                #[rustfmt::skip]
                info!("delete_inactive_final_date() time: {}", format!("{:.2?}", timer.elapsed()));
            }
            Ok(count)
        }
    }
}

#[cfg(feature = "mockdata")]
pub mod tests {
    use chrono::{DateTime, Duration, Utc};

    use crate::users::user_models::{CreateUserRecovery, UserRecovery};

    use super::{UserRecoveryOrm, DURATION_IN_DAYS};

    pub const USER_RECOVERY_ID: i32 = 1300;

    #[derive(Debug, Clone)]
    pub struct UserRecoveryOrmApp {
        pub user_recovery_vec: Vec<UserRecovery>,
    }

    impl UserRecoveryOrmApp {
        /// Create a new instance.
        pub fn new() -> Self {
            UserRecoveryOrmApp {
                user_recovery_vec: Vec::new(),
            }
        }
        /// Create a new instance with the specified user recovery list.
        #[cfg(test)]
        pub fn create(user_recov_vec: &[UserRecovery]) -> Self {
            let mut user_recovery_vec: Vec<UserRecovery> = Vec::new();
            let mut idx: i32 = 0;
            for user_reg in user_recov_vec.iter() {
                user_recovery_vec.push(Self::new_user_recovery(
                    USER_RECOVERY_ID + idx,
                    user_reg.user_id,
                    user_reg.final_date,
                ));
                idx = idx + 1;
            }
            UserRecoveryOrmApp { user_recovery_vec }
        }
        /// Create a new entity instance.
        pub fn new_user_recovery(id: i32, user_id: i32, final_date: DateTime<Utc>) -> UserRecovery {
            UserRecovery {
                id,
                user_id,
                final_date,
            }
        }
    }

    impl UserRecoveryOrm for UserRecoveryOrmApp {
        /// Get an entity (user_recovery) by ID.
        fn get_user_recovery_by_id(&self, id: i32) -> Result<Option<UserRecovery>, String> {
            let result = self
                .user_recovery_vec
                .iter()
                .find(|user_recovery| user_recovery.id == id)
                .map(|user_recovery| user_recovery.clone());
            Ok(result)
        }
        /// Find for an entity (user_recovery) by user_id.
        fn find_user_recovery_by_user_id(&self, user_id: i32) -> Result<Option<UserRecovery>, String> {
            let now = Utc::now();

            let result: Option<UserRecovery> = self
                .user_recovery_vec
                .iter()
                .find(|user_recovery| user_recovery.final_date > now && (user_recovery.user_id == user_id))
                .map(|user_recovery| user_recovery.clone());

            Ok(result)
        }

        /// Add a new entity (user_recovery).
        fn create_user_recovery(&self, create_user_recovery: CreateUserRecovery) -> Result<UserRecovery, String> {
            let user_id = create_user_recovery.user_id;
            let final_date = create_user_recovery.final_date.clone();

            let opt_res_user1: Option<UserRecovery> = self.find_user_recovery_by_user_id(user_id)?;
            if opt_res_user1.is_some() {
                return Err("\"User recovery\" already exists.".to_string());
            }

            let idx: i32 = self.user_recovery_vec.len().try_into().unwrap();
            let new_id: i32 = USER_RECOVERY_ID + idx;

            let user_recovery_saved: UserRecovery = UserRecoveryOrmApp::new_user_recovery(new_id, user_id, final_date);

            Ok(user_recovery_saved)
        }

        /// Modify an entity (user_recovery).
        fn modify_user_recovery(
            &self,
            id: i32,
            modify_profile_recovery: CreateUserRecovery,
        ) -> Result<Option<UserRecovery>, String> {
            let user_recovery_opt = self.user_recovery_vec.iter().find(|user_recovery| user_recovery.id == id);
            if user_recovery_opt.is_none() {
                return Ok(None);
            }

            let user_recovery_saved: UserRecovery = UserRecoveryOrmApp::new_user_recovery(
                id,
                modify_profile_recovery.user_id,
                modify_profile_recovery.final_date.clone(),
            );

            Ok(Some(user_recovery_saved))
        }

        /// Delete an entity (user_recovery).
        fn delete_user_recovery(&self, id: i32) -> Result<usize, String> {
            let user_recovery_opt = self.user_recovery_vec.iter().find(|user_recovery| user_recovery.id == id);

            #[rustfmt::skip]
            let result = if user_recovery_opt.is_none() { 0 } else { 1 };
            Ok(result)
        }
        /// Delete all entities (user_recovery) with an inactive "final_date".
        fn delete_inactive_final_date(&self, duration_in_days: Option<u16>) -> Result<usize, String> {
            let now = Utc::now();
            let duration = duration_in_days.unwrap_or(DURATION_IN_DAYS.into());
            let start_day_time = now - Duration::days(duration.into());
            let end_day_time = now.clone();

            let result = self
                .user_recovery_vec
                .iter()
                .filter(|user_recovery| {
                    user_recovery.final_date > start_day_time && user_recovery.final_date < end_day_time
                })
                .count();

            Ok(result)
        }
    }
}
