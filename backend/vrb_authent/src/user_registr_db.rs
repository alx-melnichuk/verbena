use vrb_db::db::DbPool2;

use crate::user_registr_models::{CreateUserRegistr, UserRegistr};

pub const DURATION_IN_DAYS: u16 = 90;

pub trait UserRegistrDb {
    /// Find for an entity (user_registration) by id.
    fn find_user_registr_by_id(&self, id: i32) -> impl std::future::Future<Output = Result<Option<UserRegistr>, String>> + Send;

    /// Find for an entity (user_registration) by nickname or email.
    fn find_user_registr_by_nickname_or_email(
        &self,
        nickname: Option<&str>,
        email: Option<&str>,
    ) -> impl std::future::Future<Output = Result<Option<UserRegistr>, String>> + Send;

    /// Add a new entity (user_registration).
    fn create_user_registr(
        &self,
        create_user_registr_dto: CreateUserRegistr,
    ) -> impl std::future::Future<Output = Result<UserRegistr, String>> + Send;

    /// Delete an entity (user_registration).
    fn delete_user_registr(&self, id: i32) -> impl std::future::Future<Output = Result<usize, String>> + Send;

    /// Delete all entities (user_registration) with an inactive "final_date".
    fn delete_inactive_final_date(&self, duration_in_days: Option<u16>) -> impl std::future::Future<Output = Result<usize, String>> + Send;
}

#[cfg(not(all(test, feature = "mockdata")))]
pub fn get_user_registr_db_app(db_pool: DbPool2) -> impls::UserRegistrDbApp {
    impls::UserRegistrDbApp::new(db_pool)
}
#[cfg(all(test, feature = "mockdata"))]
pub fn get_user_registr_db_app(_: DbPool2) -> tests::UserRegistrDbApp {
    tests::UserRegistrDbApp::new()
}

#[cfg(not(all(test, feature = "mockdata")))]
pub mod impls {
    use std::time::Instant as tm;

    use chrono::{Duration, Utc};
    use log::{Level::Info, info, log_enabled};
    use sqlx;
    use vrb_db::db::DbPool2;

    use crate::user_registr_db::{DURATION_IN_DAYS, UserRegistrDb};
    use crate::user_registr_models::{CreateUserRegistr, UserRegistr};

    pub const CONN_POOL: &str = "ConnectionPool";

    #[derive(Debug, Clone)]
    pub struct UserRegistrDbApp {
        pub db_pool: DbPool2,
    }

    impl UserRegistrDbApp {
        pub fn new(db_pool: DbPool2) -> Self {
            UserRegistrDbApp { db_pool }
        }
    }

    impl UserRegistrDb for UserRegistrDbApp {
        /// Find for an entity (user_registration) by id.
        async fn find_user_registr_by_id(&self, id: i32) -> Result<Option<UserRegistr>, String> {
            let timer = if log_enabled!(Info) { Some(tm::now()) } else { None };

            let result: Option<UserRegistr> = sqlx::query_as(
                "SELECT id, nickname, email, \"password\", final_date \
                FROM user_registration \
                WHERE id = $1 \
                LIMIT 1",
            )
            .bind(id)
            .fetch_optional(&self.db_pool)
            .await
            .map_err(|e| format!("find_user_registr_by_id: {}", e.to_string()))?;

            if let Some(timer) = timer {
                info!("find_user_registr_by_id() time: {}", format!("{:.2?}", timer.elapsed()));
            }
            Ok(result)
        }

        /// Find for an entity (user_registration) by nickname or email.
        async fn find_user_registr_by_nickname_or_email(
            &self,
            nickname: Option<&str>,
            email: Option<&str>,
        ) -> Result<Option<UserRegistr>, String> {
            let timer = if log_enabled!(Info) { Some(tm::now()) } else { None };
            let nickname2 = nickname.unwrap_or(&"".to_string()).to_lowercase();
            let nickname2_len = nickname2.len();
            let email2 = email.unwrap_or(&"".to_string()).to_lowercase();
            let email2_len = email2.len();

            if nickname2_len == 0 && email2_len == 0 {
                return Ok(None);
            }

            let mut result_vec: Vec<UserRegistr> = vec![];

            if nickname2_len > 0 && email2_len == 0 {
                let result_nickname_vec: Vec<UserRegistr> = sqlx::query_as(
                    "SELECT id, nickname, email, \"password\", final_date \
                    FROM user_registration \
                    WHERE nickname = $1 AND final_date > NOW() \
                    LIMIT 1",
                )
                .bind(nickname2)
                .fetch_all(&self.db_pool)
                .await
                .map_err(|e| format!("find_user_registr_by_nickname: {}", e.to_string()))?;

                result_vec.extend(result_nickname_vec);
            } else if nickname2_len == 0 && email2_len > 0 {
                let result_email_vec: Vec<UserRegistr> = sqlx::query_as(
                    "SELECT id, nickname, email, \"password\", final_date \
                    FROM user_registration \
                    WHERE email = $1 AND final_date > NOW() \
                    LIMIT 1",
                )
                .bind(email2)
                .fetch_all(&self.db_pool)
                .await
                .map_err(|e| format!("find_user_registr_by_email: {}", e.to_string()))?;

                result_vec.extend(result_email_vec);
            } else {
                let result_nickname_email_vec: Vec<UserRegistr> = sqlx::query_as(
                    "SELECT id, nickname, email, \"password\", final_date \
                    FROM user_registration \
                    WHERE nickname = $1 AND final_date > NOW() \
                    UNION ALL \
                    SELECT id, nickname, email, \"password\", final_date \
                    FROM user_registration \
                    WHERE email = $2 AND final_date > NOW() \
                    LIMIT 1",
                )
                .bind(nickname2)
                .bind(email2)
                .fetch_all(&self.db_pool)
                .await
                .map_err(|e| format!("find_user_registr_by_nickname_email: {}", e.to_string()))?;

                result_vec.extend(result_nickname_email_vec);
            }

            #[rustfmt::skip]
            let result = if result_vec.len() > 0 { Some(result_vec[0].clone()) } else { None };
            if let Some(timer) = timer {
                #[rustfmt::skip]
                info!("find_user_registr_by_nickname_or_email() time: {}", format!("{:.2?}", timer.elapsed()));
            }
            Ok(result)
        }

        /// Add a new entity (user_registration).
        async fn create_user_registr(&self, create_user_registr_dto: CreateUserRegistr) -> Result<UserRegistr, String> {
            let timer = if log_enabled!(Info) { Some(tm::now()) } else { None };
            let mut user_registr = create_user_registr_dto.clone();
            user_registr.nickname = user_registr.nickname.to_lowercase();
            user_registr.email = user_registr.email.to_lowercase();

            let result: UserRegistr = sqlx::query_as(
                "INSERT INTO user_registration(nickname, email, \"password\", final_date) \
                VALUES($1, $2, $3, $4) \
                RETURNING id, nickname, email, \"password\", final_date",
            )
            .bind(user_registr.nickname)
            .bind(user_registr.email)
            .bind(user_registr.password)
            .bind(user_registr.final_date)
            .fetch_one(&self.db_pool)
            .await
            .map_err(|e| format!("create_user_registr: {}", e.to_string()))?;

            if let Some(timer) = timer {
                info!("create_user_registr() time: {}", format!("{:.2?}", timer.elapsed()));
            }
            Ok(result)
        }

        /// Delete an entity (user_registration).
        async fn delete_user_registr(&self, id: i32) -> Result<usize, String> {
            let timer = if log_enabled!(Info) { Some(tm::now()) } else { None };

            let count: i64 = sqlx::query_scalar(
                "WITH deleted AS ( \
                DELETE FROM user_registration \
                WHERE id = $1 \
                RETURNING 1 \
            ) SELECT count(*) FROM deleted",
            )
            .bind(id)
            .fetch_one(&self.db_pool)
            .await
            .map_err(|e| format!("delete_user_registr: {}", e.to_string()))?;

            if let Some(timer) = timer {
                info!("delete_user_registr() time: {}", format!("{:.2?}", timer.elapsed()));
            }
            let result: usize = count.try_into().unwrap();
            Ok(result)
        }

        /// Delete all entities (user_registration) with an inactive "final_date".
        async fn delete_inactive_final_date(&self, duration_in_days: Option<u16>) -> Result<usize, String> {
            let timer = if log_enabled!(Info) { Some(tm::now()) } else { None };
            let now = Utc::now();
            let duration = duration_in_days.unwrap_or(DURATION_IN_DAYS.into());
            let start_day_time = now - Duration::days(duration.into());
            let end_day_time = now.clone();

            let count: i64 = sqlx::query_scalar(
                "WITH deleted AS ( \
                DELETE FROM user_registration \
                WHERE final_date > $1 AND final_date < $2 \
                RETURNING 1 \
            ) SELECT count(*) FROM deleted",
            )
            .bind(start_day_time)
            .bind(end_day_time)
            .fetch_one(&self.db_pool)
            .await
            .map_err(|e| format!("delete_user_registration_inactive_final_date: {}", e.to_string()))?;

            if let Some(timer) = timer {
                #[rustfmt::skip]
                info!("delete_user_registration_inactive_final_date() time: {}", format!("{:.2?}", timer.elapsed()));
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

    use crate::user_registr_db::{DURATION_IN_DAYS, UserRegistrDb};
    use crate::user_registr_models::{CreateUserRegistr, UserRegistr};

    pub const USER_REGISTR_ID: i32 = 1200;

    #[derive(Debug, Clone)]
    pub struct UserRegistrDbApp {
        pub user_registr_vec: Vec<UserRegistr>,
    }

    impl UserRegistrDbApp {
        /// Create a new instance.
        pub fn new() -> Self {
            UserRegistrDbApp {
                user_registr_vec: Vec::new(),
            }
        }
        /// Create a new instance with the specified user registr list.
        pub fn create(user_reg_vec: &[UserRegistr]) -> Self {
            let mut user_registr_vec: Vec<UserRegistr> = Vec::new();
            let mut idx: i32 = user_registr_vec.len().try_into().unwrap();
            for user_reg in user_reg_vec.iter() {
                user_registr_vec.push(Self::new_user_registr(
                    USER_REGISTR_ID + idx,
                    &user_reg.nickname,
                    &user_reg.email,
                    &user_reg.password,
                    user_reg.final_date,
                ));
                idx = idx + 1;
            }
            UserRegistrDbApp { user_registr_vec }
        }
        /// Create a new entity instance.
        pub fn new_user_registr(id: i32, nickname: &str, email: &str, password: &str, final_date: DateTime<Utc>) -> UserRegistr {
            UserRegistr {
                id,
                nickname: nickname.to_lowercase(),
                email: email.to_lowercase(),
                password: password.to_string(),
                final_date: final_date,
            }
        }
    }

    impl UserRegistrDb for UserRegistrDbApp {
        /// Find for an entity (user_registration) by id.
        async fn find_user_registr_by_id(&self, id: i32) -> Result<Option<UserRegistr>, String> {
            let result = self
                .user_registr_vec
                .iter()
                .find(|user_registr| user_registr.id == id)
                .map(|user_registr| user_registr.clone());
            Ok(result)
        }
        /// Find for an entity (user_registration) by nickname or email.
        async fn find_user_registr_by_nickname_or_email(
            &self,
            nickname: Option<&str>,
            email: Option<&str>,
        ) -> Result<Option<UserRegistr>, String> {
            let nickname2 = nickname.unwrap_or(&"".to_string()).to_lowercase();
            let nickname2_len = nickname2.len();
            let email2 = email.unwrap_or(&"".to_string()).to_lowercase();
            let email2_len = email2.len();

            if nickname2_len == 0 && email2_len == 0 {
                return Ok(None);
            }

            let now = Utc::now();
            #[rustfmt::skip]
            let result: Option<UserRegistr> = self
                .user_registr_vec
                .iter()
                .find(|user_registr|
                    user_registr.final_date > now
                        && (user_registr.nickname == nickname2 || user_registr.email == email2))
                .map(|user_registr| user_registr.clone());

            Ok(result)
        }
        /// Add a new entity (user_registration).
        async fn create_user_registr(&self, create_user_registr: CreateUserRegistr) -> Result<UserRegistr, String> {
            let nickname = create_user_registr.nickname.clone();
            let email = create_user_registr.email.clone();
            let password = create_user_registr.password.clone();
            let final_date = create_user_registr.final_date.clone();
            #[rustfmt::skip]
            let opt_res_user1: Option<UserRegistr> =
                self.find_user_registr_by_nickname_or_email(Some(&nickname), Some(&email)).await?;
            if opt_res_user1.is_some() {
                return Err("\"User Registration\" already exists.".to_string());
            }

            let idx: i32 = self.user_registr_vec.len().try_into().unwrap();
            let new_id: i32 = USER_REGISTR_ID + idx;
            let nickname = create_user_registr.nickname.clone();
            let email = create_user_registr.email.clone();
            #[rustfmt::skip]
            let user_registr_saved: UserRegistr =
                UserRegistrDbApp::new_user_registr(new_id, &nickname, &email, &password, final_date);

            Ok(user_registr_saved)
        }
        /// Delete an entity (user_registration).
        async fn delete_user_registr(&self, id: i32) -> Result<usize, String> {
            #[rustfmt::skip]
            let opt_user_registr: Option<&UserRegistr> =
                self.user_registr_vec.iter().find(|user_registr| user_registr.id == id);

            #[rustfmt::skip]
            let result = if opt_user_registr.is_none() { 0 } else { 1 };
            Ok(result)
        }
        /// Delete all entities (user_registration) with an inactive "final_date".
        async fn delete_inactive_final_date(&self, duration_in_days: Option<u16>) -> Result<usize, String> {
            let now = Utc::now();
            let duration = duration_in_days.unwrap_or(DURATION_IN_DAYS.into());
            let start_day_time = now - Duration::days(duration.into());
            let end_day_time = now.clone();
            #[rustfmt::skip]
            let result = self
                .user_registr_vec
                .iter()
                .filter(|user_registr|
                    user_registr.final_date > start_day_time && user_registr.final_date < end_day_time)
                .count();

            Ok(result)
        }
    }

    pub struct UserRegistrDbTest {}

    impl UserRegistrDbTest {
        pub fn registrs(is_exist: bool) -> Vec<UserRegistr> {
            let user_registr_vec: Vec<UserRegistr> = match is_exist {
                true => {
                    let id: i32 = USER_REGISTR_ID;
                    let nickname = "robert_brown";
                    let email = format!("{}@gmail.com", nickname);
                    let final_date: DateTime<Utc> = Utc::now() + Duration::minutes(20);
                    let user_registr = UserRegistrDbApp::new_user_registr(id, nickname, &email, "passwdR2B2", final_date);
                    UserRegistrDbApp::create(&[user_registr]).user_registr_vec
                }
                false => vec![],
            };
            user_registr_vec
        }
        pub fn cfg_registr_db(registr: Vec<UserRegistr>) -> impl FnOnce(&mut web::ServiceConfig) {
            move |config: &mut web::ServiceConfig| {
                let data_user_registr_db = web::Data::new(UserRegistrDbApp::create(&registr));
                config.app_data(web::Data::clone(&data_user_registr_db));
            }
        }
    }
}
