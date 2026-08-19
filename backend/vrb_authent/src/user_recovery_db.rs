use vrb_db::db::DbPool2;

use crate::user_recovery_models::{CreateUserRecovery, UserRecovery};

pub const DURATION_IN_DAYS: u16 = 90;

pub trait UserRecoveryDb {
    /// Get an entity (user_recovery) by ID.
    fn get_user_recovery_by_id(&self, id: i32) -> impl std::future::Future<Output = Result<Option<UserRecovery>, String>> + Send;

    /// Find for an entity (user_recovery) by user_id.
    fn find_user_recovery_by_user_id(&self, user_id: i32)
    -> impl std::future::Future<Output = Result<Option<UserRecovery>, String>> + Send;

    /// Add a new entity (user_recovery).
    fn create_user_recovery(
        &self,
        create_user_recovery: CreateUserRecovery,
    ) -> impl std::future::Future<Output = Result<UserRecovery, String>> + Send;

    /// Modify an entity (user_recovery).
    fn modify_user_recovery(
        &self,
        id: i32,
        modify_user_recovery: CreateUserRecovery,
    ) -> impl std::future::Future<Output = Result<Option<UserRecovery>, String>> + Send;

    /// Delete an entity (user_recovery).
    fn delete_user_recovery(&self, id: i32) -> impl std::future::Future<Output = Result<usize, String>> + Send;

    /// Delete all entities (user_recovery) with an inactive "final_date".
    fn delete_inactive_final_date(&self, duration_in_days: Option<u16>) -> impl std::future::Future<Output = Result<usize, String>> + Send;
}

#[cfg(not(all(test, feature = "mockdata")))]
pub fn get_user_recovery_db_app(db_pool: DbPool2) -> impls::UserRecoveryDbApp {
    impls::UserRecoveryDbApp::new(db_pool)
}
#[cfg(all(test, feature = "mockdata"))]
pub fn get_user_recovery_db_app(_: DbPool2) -> tests::UserRecoveryDbApp {
    tests::UserRecoveryDbApp::new()
}

#[cfg(not(all(test, feature = "mockdata")))]
pub mod impls {
    use std::time::Instant as tm;

    use chrono::{Duration, Utc};
    use log::{Level::Info, info, log_enabled};
    use sqlx;
    use vrb_db::db::DbPool2;

    use crate::user_recovery_db::DURATION_IN_DAYS;
    use crate::user_recovery_models::UserRecovery;

    use super::*;

    pub const CONN_POOL: &str = "ConnectionPool";

    #[derive(Debug, Clone)]
    pub struct UserRecoveryDbApp {
        pub db_pool: DbPool2,
    }

    impl UserRecoveryDbApp {
        pub fn new(db_pool: DbPool2) -> Self {
            UserRecoveryDbApp { db_pool }
        }
    }

    impl UserRecoveryDb for UserRecoveryDbApp {
        /// Get an entity (user_recovery) by ID.
        async fn get_user_recovery_by_id(&self, id: i32) -> Result<Option<UserRecovery>, String> {
            let timer = if log_enabled!(Info) { Some(tm::now()) } else { None };

            let result: Option<UserRecovery> = sqlx::query_as(
                "SELECT id, user_id, final_date \
                FROM user_recovery \
                WHERE id = $1 \
                LIMIT 1",
            )
            .bind(id)
            .fetch_optional(&self.db_pool)
            .await
            .map_err(|e| format!("find_user_recovery_by_id: {}", e.to_string()))?;

            if let Some(timer) = timer {
                info!("get_user_recovery_by_id() time: {}", format!("{:.2?}", timer.elapsed()));
            }
            Ok(result)
        }

        /// Find for an entity (user_recovery) by user_id.
        async fn find_user_recovery_by_user_id(&self, user_id: i32) -> Result<Option<UserRecovery>, String> {
            let timer = if log_enabled!(Info) { Some(tm::now()) } else { None };

            let result: Option<UserRecovery> = sqlx::query_as(
                "SELECT id, user_id, final_date \
                FROM user_recovery \
                WHERE user_id = $1 AND final_date > NOW() \
                LIMIT 1",
            )
            .bind(user_id)
            .fetch_optional(&self.db_pool)
            .await
            .map_err(|e| format!("find_user_recovery_by_user_id: {}", e.to_string()))?;

            if let Some(timer) = timer {
                #[rustfmt::skip]
                info!("find_user_recovery_by_user_id() time: {}", format!("{:.2?}", timer.elapsed()));
            }
            Ok(result)
        }

        /// Add a new entity (user_recovery).
        async fn create_user_recovery(&self, create_user_recovery: CreateUserRecovery) -> Result<UserRecovery, String> {
            let timer = if log_enabled!(Info) { Some(tm::now()) } else { None };

            let result: UserRecovery = sqlx::query_as(
                "INSERT INTO user_recovery(user_id, final_date) \
                VALUES($1, $2) \
                RETURNING id, user_id, final_date",
            )
            .bind(create_user_recovery.user_id)
            .bind(create_user_recovery.final_date)
            .fetch_one(&self.db_pool)
            .await
            .map_err(|e| format!("create_user_recovery: {}", e.to_string()))?;

            if let Some(timer) = timer {
                info!("create_user_recovery() time: {}", format!("{:.2?}", timer.elapsed()));
            }
            Ok(result)
        }

        /// Modify an entity (user_recovery).
        async fn modify_user_recovery(&self, id: i32, create_user_recovery: CreateUserRecovery) -> Result<Option<UserRecovery>, String> {
            let timer = if log_enabled!(Info) { Some(tm::now()) } else { None };

            let result: Option<UserRecovery> = sqlx::query_as(
                "UPDATE user_recovery SET \
                user_id=$2, final_date=$3 \
                WHERE id=$1 \
                RETURNING id, user_id, final_date",
            )
            .bind(id)
            .bind(create_user_recovery.user_id)
            .bind(create_user_recovery.final_date)
            .fetch_optional(&self.db_pool)
            .await
            .map_err(|e| format!("modify_user_recovery: {}", e.to_string()))?;

            if let Some(timer) = timer {
                info!("modify_user_recovery() time: {}", format!("{:.2?}", timer.elapsed()));
            }
            Ok(result)
        }

        /// Delete an entity (user_recovery).
        async fn delete_user_recovery(&self, id: i32) -> Result<usize, String> {
            let timer = if log_enabled!(Info) { Some(tm::now()) } else { None };

            let count: i64 = sqlx::query_scalar(
                "WITH deleted AS ( \
                DELETE FROM user_recovery \
                WHERE id = $1 \
                RETURNING 1 \
            ) SELECT count(*) FROM deleted",
            )
            .bind(id)
            .fetch_one(&self.db_pool)
            .await
            .map_err(|e| format!("delete_user_recovery: {}", e.to_string()))?;

            if let Some(timer) = timer {
                info!("delete_user_recovery() time: {}", format!("{:.2?}", timer.elapsed()));
            }
            let result: usize = count.try_into().unwrap();
            Ok(result)
        }

        /// Delete all entities (user_recovery) with an inactive "final_date".
        async fn delete_inactive_final_date(&self, duration_in_days: Option<u16>) -> Result<usize, String> {
            let timer = if log_enabled!(Info) { Some(tm::now()) } else { None };

            let now = Utc::now();
            let duration = duration_in_days.unwrap_or(DURATION_IN_DAYS.into());
            let start_day_time = now - Duration::days(duration.into());
            let end_day_time = now.clone();

            let count: i64 = sqlx::query_scalar(
                "WITH deleted AS ( \
                DELETE FROM user_recovery \
                WHERE final_date > $1 AND final_date < $2 \
                RETURNING 1 \
            ) SELECT count(*) FROM deleted",
            )
            .bind(start_day_time)
            .bind(end_day_time)
            .fetch_one(&self.db_pool)
            .await
            .map_err(|e| format!("delete_user_recovery_inactive_final_date: {}", e.to_string()))?;

            if let Some(timer) = timer {
                #[rustfmt::skip]
                info!("delete_user_recovery_inactive_final_date() time: {}", format!("{:.2?}", timer.elapsed()));
            }
            let result: usize = count.try_into().unwrap();
            Ok(result)
        }
    }
}

#[cfg(any(test, feature = "mockdata"))]
pub mod tests {
    use actix_web::web;
    use chrono::{DateTime, Duration, Utc};

    use crate::user_recovery_db::{DURATION_IN_DAYS, UserRecoveryDb};
    use crate::user_recovery_models::{CreateUserRecovery, UserRecovery};

    pub const USER_RECOVERY_ID: i32 = 1300;

    #[derive(Debug, Clone)]
    pub struct UserRecoveryDbApp {
        pub user_recovery_vec: Vec<UserRecovery>,
    }

    impl UserRecoveryDbApp {
        /// Create a new instance.
        pub fn new() -> Self {
            UserRecoveryDbApp {
                user_recovery_vec: Vec::new(),
            }
        }
        /// Create a new instance with the specified user recovery list.
        pub fn create(user_recov_vec: &[UserRecovery]) -> Self {
            let mut user_recovery_vec: Vec<UserRecovery> = Vec::new();
            let mut idx: i32 = 0;
            for user_reg in user_recov_vec.iter() {
                #[rustfmt::skip]
                user_recovery_vec.push(Self::new_user_recovery(
                    USER_RECOVERY_ID + idx, user_reg.user_id, user_reg.final_date));
                idx = idx + 1;
            }
            UserRecoveryDbApp { user_recovery_vec }
        }
        /// Create a new entity instance.
        pub fn new_user_recovery(id: i32, user_id: i32, final_date: DateTime<Utc>) -> UserRecovery {
            UserRecovery { id, user_id, final_date }
        }
    }

    impl UserRecoveryDb for UserRecoveryDbApp {
        /// Get an entity (user_recovery) by ID.
        async fn get_user_recovery_by_id(&self, id: i32) -> Result<Option<UserRecovery>, String> {
            let result = self
                .user_recovery_vec
                .iter()
                .find(|user_recovery| user_recovery.id == id)
                .map(|user_recovery| user_recovery.clone());
            Ok(result)
        }

        /// Find for an entity (user_recovery) by user_id.
        async fn find_user_recovery_by_user_id(&self, user_id: i32) -> Result<Option<UserRecovery>, String> {
            let now = Utc::now();

            let result: Option<UserRecovery> = self
                .user_recovery_vec
                .iter()
                .find(|user_recovery| user_recovery.final_date > now && (user_recovery.user_id == user_id))
                .map(|user_recovery| user_recovery.clone());

            Ok(result)
        }

        /// Add a new entity (user_recovery).
        async fn create_user_recovery(&self, create_user_recovery: CreateUserRecovery) -> Result<UserRecovery, String> {
            let user_id = create_user_recovery.user_id;
            let final_date = create_user_recovery.final_date.clone();

            let opt_res_user1: Option<UserRecovery> = self.find_user_recovery_by_user_id(user_id).await?;
            if opt_res_user1.is_some() {
                return Err("\"User recovery\" already exists.".to_string());
            }

            let idx: i32 = self.user_recovery_vec.len().try_into().unwrap();
            let new_id: i32 = USER_RECOVERY_ID + idx;

            let user_recovery_saved: UserRecovery = UserRecoveryDbApp::new_user_recovery(new_id, user_id, final_date);

            Ok(user_recovery_saved)
        }

        /// Modify an entity (user_recovery).
        async fn modify_user_recovery(&self, id: i32, modify_profile_recovery: CreateUserRecovery) -> Result<Option<UserRecovery>, String> {
            let user_recovery_opt = self.user_recovery_vec.iter().find(|user_recovery| user_recovery.id == id);
            if user_recovery_opt.is_none() {
                return Ok(None);
            }

            let user_recovery_saved: UserRecovery =
                UserRecoveryDbApp::new_user_recovery(id, modify_profile_recovery.user_id, modify_profile_recovery.final_date.clone());

            Ok(Some(user_recovery_saved))
        }

        /// Delete an entity (user_recovery).
        async fn delete_user_recovery(&self, id: i32) -> Result<usize, String> {
            let user_recovery_opt = self.user_recovery_vec.iter().find(|user_recovery| user_recovery.id == id);

            #[rustfmt::skip]
            let result = if user_recovery_opt.is_none() { 0 } else { 1 };
            Ok(result)
        }

        /// Delete all entities (user_recovery) with an inactive "final_date".
        async fn delete_inactive_final_date(&self, duration_in_days: Option<u16>) -> Result<usize, String> {
            let now = Utc::now();
            let duration = duration_in_days.unwrap_or(DURATION_IN_DAYS.into());
            let start_day_time = now - Duration::days(duration.into());
            let end_day_time = now.clone();

            let result = self
                .user_recovery_vec
                .iter()
                .filter(|user_recovery| user_recovery.final_date > start_day_time && user_recovery.final_date < end_day_time)
                .count();

            Ok(result)
        }
    }

    pub struct UserRecoveryDbTest {}

    impl UserRecoveryDbTest {
        pub fn recoveries(opt_user_id: Option<i32>) -> Vec<UserRecovery> {
            let user_recovery_vec: Vec<UserRecovery> = match opt_user_id {
                Some(user_id) => {
                    let final_date_utc = Utc::now() + Duration::seconds(600);
                    let user_recovery = UserRecoveryDbApp::new_user_recovery(1, user_id, final_date_utc);
                    UserRecoveryDbApp::create(&vec![user_recovery]).user_recovery_vec
                }
                None => vec![],
            };
            user_recovery_vec
        }
        pub fn cfg_recovery_db(recovery: Vec<UserRecovery>) -> impl FnOnce(&mut web::ServiceConfig) {
            move |config: &mut web::ServiceConfig| {
                let data_user_recovery_db = web::Data::new(UserRecoveryDbApp::create(&recovery));
                config.app_data(web::Data::clone(&data_user_recovery_db));
            }
        }
    }
}
