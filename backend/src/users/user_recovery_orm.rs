use super::user_models::{CreateUserRecoveryDto, UserRecovery};

pub trait UserRecoveryOrm {
    /// Find for an entity (user_recovery) by id.
    fn find_user_recovery_by_id(&self, id: i32) -> Result<Option<UserRecovery>, String>;
    /// Find for an entity (user_recovery) by user_id.
    fn find_user_recovery_by_user_id(&self, user_id: i32) -> Result<Option<UserRecovery>, String>;
    /// Add a new entity (user_recovery).
    fn create_user_recovery(
        &self,
        create_user_recovery_dto: &CreateUserRecoveryDto,
    ) -> Result<UserRecovery, String>;
    /// Modify an entity (user_recovery).
    fn modify_user_recovery(
        &self,
        id: i32,
        modify_user_recovery_dto: &CreateUserRecoveryDto,
    ) -> Result<Option<UserRecovery>, String>;
    /// Delete an entity (user_recovery).
    fn delete_user_recovery(&self, id: i32) -> Result<usize, String>;
    /// Delete all entities (user_recovery) with an inactive "final_date".
    fn delete_inactive_final_date(&self) -> Result<usize, String>;
}

pub mod cfg {
    use crate::dbase::DbPool;

    #[cfg(not(feature = "mockdata"))]
    use super::inst::UserRecoveryOrmApp;
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
pub mod inst {

    use chrono::Utc;
    // use diesel::{debug_query, pg::Pg};
    use diesel::{self, prelude::*};

    use crate::dbase;
    use crate::schema;
    use crate::users::user_models::{CreateUserRecoveryDto, UserRecovery};

    use super::UserRecoveryOrm;

    pub const CONN_POOL: &str = "ConnectionPool";
    pub const DB_USER_RECOVERY: &str = "Db_UserRecovery";

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
        /// Find for an entity (user_recovery) by id.
        fn find_user_recovery_by_id(&self, id: i32) -> Result<Option<UserRecovery>, String> {
            // Get a connection from the P2D2 pool.
            let mut conn = self.get_conn()?;
            // Run query using Diesel to find user by id and return it.
            let result = schema::user_recovery::table
                .filter(schema::user_recovery::dsl::id.eq(id))
                .first::<UserRecovery>(&mut conn)
                .optional()
                .map_err(|e| format!("{DB_USER_RECOVERY}: {}", e.to_string()))?;

            Ok(result)
        }

        /// Find for an entity (user_recovery) by user_id.
        fn find_user_recovery_by_user_id(
            &self,
            user_id: i32,
        ) -> Result<Option<UserRecovery>, String> {
            // Get a connection from the P2D2 pool.
            let mut conn = self.get_conn()?;
            let today = Utc::now();
            // Run query using Diesel to find user by user_id and return it (where final_date > now).
            let result = schema::user_recovery::table
                .filter(
                    schema::user_recovery::dsl::user_id
                        .eq(user_id)
                        .and(schema::user_recovery::dsl::final_date.gt(today)),
                )
                .first::<UserRecovery>(&mut conn)
                .optional()
                .map_err(|e| format!("{DB_USER_RECOVERY}: {}", e.to_string()))?;

            Ok(result)
        }

        /// Add a new entity (user_recovery).
        fn create_user_recovery(
            &self,
            create_user_recovery_dto: &CreateUserRecoveryDto,
        ) -> Result<UserRecovery, String> {
            let create_user_recovery_dto2 = create_user_recovery_dto.clone();

            // Get a connection from the P2D2 pool.
            let mut conn = self.get_conn()?;
            // Run query using Diesel to add a new user entry.
            let user_recovery: UserRecovery = diesel::insert_into(schema::user_recovery::table)
                .values(create_user_recovery_dto2)
                .returning(UserRecovery::as_returning())
                .get_result(&mut conn)
                .map_err(|e| format!("{DB_USER_RECOVERY}: {}", e.to_string()))?;

            Ok(user_recovery)
        }

        /// Modify an entity (user_recovery).
        fn modify_user_recovery(
            &self,
            id: i32,
            create_user_recovery_dto: &CreateUserRecoveryDto,
        ) -> Result<Option<UserRecovery>, String> {
            let create_user_recovery_dto2 = create_user_recovery_dto.clone();

            // Get a connection from the P2D2 pool.
            let mut conn = self.get_conn()?;
            // Run query using Diesel to full or partially modify the user entry.
            let result = diesel::update(schema::user_recovery::dsl::user_recovery.find(id))
                .set(&create_user_recovery_dto2)
                .returning(UserRecovery::as_returning())
                .get_result(&mut conn)
                .optional()
                .map_err(|e| format!("{DB_USER_RECOVERY}: {}", e.to_string()))?;

            Ok(result)
        }

        /// Delete an entity (user_recovery).
        fn delete_user_recovery(&self, id: i32) -> Result<usize, String> {
            // Get a connection from the P2D2 pool.
            let mut conn = self.get_conn()?;
            // Run query using Diesel to delete a entry (user_recovery).
            let count: usize = diesel::delete(schema::user_recovery::dsl::user_recovery.find(id))
                .execute(&mut conn)
                .map_err(|e| format!("{DB_USER_RECOVERY}: {}", e.to_string()))?;

            Ok(count)
        }

        /// Delete all entities (user_recovery) with an inactive "final_date".
        fn delete_inactive_final_date(&self) -> Result<usize, String> {
            let today = Utc::now();

            // Get a connection from the P2D2 pool.
            let mut conn = self.get_conn()?;
            // Run query using Diesel to delete a entry (user_recovery).
            let count: usize = diesel::delete(
                schema::user_recovery::table
                    .filter(schema::user_recovery::dsl::final_date.lt(today)),
            )
            .execute(&mut conn)
            .map_err(|e| format!("{DB_USER_RECOVERY}: {}", e.to_string()))?;

            Ok(count)
        }
    }
}

#[cfg(feature = "mockdata")]
pub mod tests {
    use chrono::{DateTime, Utc};

    use crate::users::user_models::{CreateUserRecoveryDto, UserRecovery};

    use super::UserRecoveryOrm;

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
        pub fn create(user_recov_vec: Vec<UserRecovery>) -> Self {
            let mut user_recovery_vec: Vec<UserRecovery> = Vec::new();
            let mut idx: i32 = user_recov_vec.len().try_into().unwrap();
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
        /// Find for an entity (user_recovery) by id.
        fn find_user_recovery_by_id(&self, id: i32) -> Result<Option<UserRecovery>, String> {
            let result = self
                .user_recovery_vec
                .iter()
                .find(|user_recovery| user_recovery.id == id)
                .map(|user_recovery| user_recovery.clone());
            Ok(result)
        }
        /// Find for an entity (user_recovery) by user_id.
        fn find_user_recovery_by_user_id(
            &self,
            user_id: i32,
        ) -> Result<Option<UserRecovery>, String> {
            let today = Utc::now();

            let result: Option<UserRecovery> = self
                .user_recovery_vec
                .iter()
                .find(|user_recovery| {
                    user_recovery.final_date > today && (user_recovery.user_id == user_id)
                })
                .map(|user_recovery| user_recovery.clone());

            Ok(result)
        }

        /// Add a new entity (user_recovery).
        fn create_user_recovery(
            &self,
            create_user_recovery_dto: &CreateUserRecoveryDto,
        ) -> Result<UserRecovery, String> {
            let user_id = create_user_recovery_dto.user_id;
            let final_date = create_user_recovery_dto.final_date.clone();

            let res_user1_opt: Option<UserRecovery> =
                self.find_user_recovery_by_user_id(user_id)?;
            if res_user1_opt.is_some() {
                return Err("\"User recovery\" already exists.".to_string());
            }

            let idx: i32 = self.user_recovery_vec.len().try_into().unwrap();
            let new_id: i32 = USER_RECOVERY_ID + idx;

            let user_recovery_saved: UserRecovery =
                UserRecoveryOrmApp::new_user_recovery(new_id, user_id, final_date);

            Ok(user_recovery_saved)
        }

        /// Modify an entity (user_recovery).
        fn modify_user_recovery(
            &self,
            id: i32,
            create_user_recovery_dto: &CreateUserRecoveryDto,
        ) -> Result<Option<UserRecovery>, String> {
            let user_recovery_opt =
                self.user_recovery_vec.iter().find(|user_recovery| user_recovery.id == id);
            if user_recovery_opt.is_none() {
                return Ok(None);
            }

            let user_recovery_saved: UserRecovery = UserRecoveryOrmApp::new_user_recovery(
                id,
                create_user_recovery_dto.user_id,
                create_user_recovery_dto.final_date.clone(),
            );

            Ok(Some(user_recovery_saved))
        }

        /// Delete an entity (user_recovery).
        fn delete_user_recovery(&self, id: i32) -> Result<usize, String> {
            let exist_user_recovery_opt =
                self.user_recovery_vec.iter().find(|user_recovery| user_recovery.id == id);

            if exist_user_recovery_opt.is_none() {
                Ok(0)
            } else {
                Ok(1)
            }
        }
        /// Delete all entities (user_recovery) with an inactive "final_date".
        fn delete_inactive_final_date(&self) -> Result<usize, String> {
            let today = Utc::now();

            let result = self
                .user_recovery_vec
                .iter()
                .filter(|user_recovery| user_recovery.final_date < today)
                .count();

            Ok(result)
        }
    }
}
