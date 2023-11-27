use super::user_registr_models::{CreateUserRegistrDto, UserRegistr};

pub const DURATION_IN_DAYS: u16 = 90;

pub trait UserRegistrOrm {
    /// Find for an entity (user_registration) by id.
    fn find_user_registr_by_id(&self, id: i32) -> Result<Option<UserRegistr>, String>;
    /// Find for an entity (user_registration) by nickname or email.
    fn find_user_registr_by_nickname_or_email(
        &self,
        nickname: Option<&str>,
        email: Option<&str>,
    ) -> Result<Option<UserRegistr>, String>;
    /// Add a new entity (user_registration).
    fn create_user_registr(
        &self,
        create_user_registr_dto: &CreateUserRegistrDto,
    ) -> Result<UserRegistr, String>;
    /// Delete an entity (user_registration).
    fn delete_user_registr(&self, id: i32) -> Result<usize, String>;
    /// Delete all entities (user_registration) with an inactive "final_date".
    fn delete_inactive_final_date(&self, duration_in_days: Option<u16>) -> Result<usize, String>;
}

pub mod cfg {
    use crate::dbase::DbPool;

    #[cfg(not(feature = "mockdata"))]
    use super::inst::UserRegistrOrmApp;
    #[cfg(not(feature = "mockdata"))]
    pub fn get_user_registr_orm_app(pool: DbPool) -> UserRegistrOrmApp {
        UserRegistrOrmApp::new(pool)
    }

    #[cfg(feature = "mockdata")]
    use super::tests::UserRegistrOrmApp;
    #[cfg(feature = "mockdata")]
    pub fn get_user_registr_orm_app(_: DbPool) -> UserRegistrOrmApp {
        UserRegistrOrmApp::new()
    }
}

#[cfg(not(feature = "mockdata"))]
pub mod inst {

    use chrono::{Duration, Utc};
    use diesel::{self, prelude::*};
    use schema::user_registration::dsl;

    use crate::dbase;
    use crate::schema;
    use crate::users::{
        user_registr_models::{CreateUserRegistrDto, UserRegistr},
        user_registr_orm::DURATION_IN_DAYS,
    };

    use super::UserRegistrOrm;

    pub const CONN_POOL: &str = "ConnectionPool";
    pub const DB_USER_REGISTR: &str = "Db_UserRegistr";

    #[derive(Debug, Clone)]
    pub struct UserRegistrOrmApp {
        pub pool: dbase::DbPool,
    }

    impl UserRegistrOrmApp {
        pub fn new(pool: dbase::DbPool) -> Self {
            UserRegistrOrmApp { pool }
        }
        pub fn get_conn(&self) -> Result<dbase::DbPooledConnection, String> {
            (&self.pool).get().map_err(|e| format!("{CONN_POOL}: {}", e.to_string()))
        }
    }

    impl UserRegistrOrm for UserRegistrOrmApp {
        /// Find for an entity (user_registration) by id.
        fn find_user_registr_by_id(&self, id: i32) -> Result<Option<UserRegistr>, String> {
            // Get a connection from the P2D2 pool.
            let mut conn = self.get_conn()?;
            // Run query using Diesel to find user by id and return it.
            let result = schema::user_registration::table
                .filter(dsl::id.eq(id))
                .first::<UserRegistr>(&mut conn)
                .optional()
                .map_err(|e| format!("{}: {}", DB_USER_REGISTR, e.to_string()))?;

            Ok(result)
        }

        /// Find for an entity (user_registration) by nickname or email.
        fn find_user_registr_by_nickname_or_email(
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

            // Get a connection from the P2D2 pool.
            let mut conn = self.get_conn()?;
            let now = Utc::now();

            // Run query using Diesel to find user by nickname and return it (where final_date > now).
            let sql_query_nickname = schema::user_registration::table
                .filter(dsl::nickname.eq(nickname2).and(dsl::final_date.gt(now)))
                .select(schema::user_registration::all_columns)
                .limit(1);
            // Run query using Diesel to find user by email and return it (where final_date > now).
            let sql_query_email = schema::user_registration::table
                .filter(dsl::email.eq(email2).and(dsl::final_date.gt(now)))
                .select(schema::user_registration::all_columns)
                .limit(1);

            let mut result_vec: Vec<UserRegistr> = vec![];

            if nickname2_len > 0 && email2_len == 0 {
                let result_nickname_vec: Vec<UserRegistr> = sql_query_nickname
                    .load(&mut conn)
                    .map_err(|e| format!("{}: {}", DB_USER_REGISTR, e.to_string()))?;
                result_vec.extend(result_nickname_vec);
            } else if nickname2_len == 0 && email2_len > 0 {
                let result_email_vec: Vec<UserRegistr> = sql_query_email
                    .load(&mut conn)
                    .map_err(|e| format!("{}: {}", DB_USER_REGISTR, e.to_string()))?;
                result_vec.extend(result_email_vec);
            } else {
                // This design (union two queries) allows the use of two separate indexes.
                let sql_query = sql_query_nickname.union_all(sql_query_email);
                // eprintln!("#sql_query: `{}`", debug_query::<Pg, _>(&sql_query).to_string());
                // Run query using Diesel to find user by nickname or email and return it (where final_date > now).
                let result_nickname_email_vec: Vec<UserRegistr> = sql_query
                    .load(&mut conn)
                    .map_err(|e| format!("{}: {}", DB_USER_REGISTR, e.to_string()))?;
                result_vec.extend(result_nickname_email_vec);
            }

            let result = if result_vec.len() > 0 {
                Some(result_vec[0].clone())
            } else {
                None
            };
            Ok(result)
        }

        /// Add a new entity (user_registration).
        fn create_user_registr(
            &self,
            create_user_registr_dto: &CreateUserRegistrDto,
        ) -> Result<UserRegistr, String> {
            let mut create_user_registr_dto2 = create_user_registr_dto.clone();
            create_user_registr_dto2.nickname = create_user_registr_dto2.nickname.to_lowercase();
            create_user_registr_dto2.email = create_user_registr_dto2.email.to_lowercase();

            // Get a connection from the P2D2 pool.
            let mut conn = self.get_conn()?;
            // Run query using Diesel to add a new user entry.
            let user_registr: UserRegistr = diesel::insert_into(schema::user_registration::table)
                .values(create_user_registr_dto2)
                .returning(UserRegistr::as_returning())
                .get_result(&mut conn)
                .map_err(|e| format!("{}: {}", DB_USER_REGISTR, e.to_string()))?;

            Ok(user_registr)
        }

        /// Delete an entity (user_registration).
        fn delete_user_registr(&self, id: i32) -> Result<usize, String> {
            // Get a connection from the P2D2 pool.
            let mut conn = self.get_conn()?;
            // Run query using Diesel to delete a entry (user_registration).
            let count: usize = diesel::delete(dsl::user_registration.find(id))
                .execute(&mut conn)
                .map_err(|e| format!("{}: {}", DB_USER_REGISTR, e.to_string()))?;

            Ok(count)
        }

        /// Delete all entities (user_registration) with an inactive "final_date".
        fn delete_inactive_final_date(
            &self,
            duration_in_days: Option<u16>,
        ) -> Result<usize, String> {
            let now = Utc::now();
            let duration = duration_in_days.unwrap_or(DURATION_IN_DAYS.into());
            let start_day_time = now - Duration::days(duration.into());
            let end_day_time = now.clone();
            let before = std::time::Instant::now();
            // Get a connection from the P2D2 pool.
            let mut conn = self.get_conn()?;

            // Run query using Diesel to delete a entry (user_registration).
            let count: usize =
                diesel::delete(schema::user_registration::table.filter(
                    dsl::final_date.gt(start_day_time).and(dsl::final_date.lt(end_day_time)),
                ))
                .execute(&mut conn)
                .map_err(|e| format!("{}: {}", DB_USER_REGISTR, e.to_string()))?;

            let info = format!("{:.2?}", before.elapsed());
            #[rustfmt::skip]
            log::info!("user_registration.delete(expired) time: {}, count: {}", info, count);

            Ok(count)
        }
    }
}

#[cfg(feature = "mockdata")]
pub mod tests {
    use chrono::{DateTime, Duration, Utc};

    use crate::users::user_registr_models::{CreateUserRegistrDto, UserRegistr};

    use super::{UserRegistrOrm, DURATION_IN_DAYS};

    pub const USER_REGISTR_ID: i32 = 1200;

    #[derive(Debug, Clone)]
    pub struct UserRegistrOrmApp {
        pub user_registr_vec: Vec<UserRegistr>,
    }

    impl UserRegistrOrmApp {
        /// Create a new instance.
        pub fn new() -> Self {
            UserRegistrOrmApp {
                user_registr_vec: Vec::new(),
            }
        }
        /// Create a new instance with the specified user registr list.
        #[cfg(test)]
        pub fn create(user_reg_vec: Vec<UserRegistr>) -> Self {
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
            UserRegistrOrmApp { user_registr_vec }
        }
        /// Create a new entity instance.
        pub fn new_user_registr(
            id: i32,
            nickname: &str,
            email: &str,
            password: &str,
            final_date: DateTime<Utc>,
        ) -> UserRegistr {
            UserRegistr {
                id,
                nickname: nickname.to_lowercase(),
                email: email.to_lowercase(),
                password: password.to_string(),
                final_date: final_date,
            }
        }
    }

    impl UserRegistrOrm for UserRegistrOrmApp {
        /// Find for an entity (user_registration) by id.
        fn find_user_registr_by_id(&self, id: i32) -> Result<Option<UserRegistr>, String> {
            let result = self
                .user_registr_vec
                .iter()
                .find(|user_registr| user_registr.id == id)
                .map(|user_registr| user_registr.clone());
            Ok(result)
        }
        /// Find for an entity (user_registration) by nickname or email.
        fn find_user_registr_by_nickname_or_email(
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

            let result: Option<UserRegistr> = self
                .user_registr_vec
                .iter()
                .find(|user_registr| {
                    user_registr.final_date > now
                        && (user_registr.nickname == nickname2 || user_registr.email == email2)
                })
                .map(|user_registr| user_registr.clone());

            Ok(result)
        }
        /// Add a new entity (user_registration).
        fn create_user_registr(
            &self,
            create_user_registr_dto: &CreateUserRegistrDto,
        ) -> Result<UserRegistr, String> {
            let nickname = create_user_registr_dto.nickname.clone();
            let email = create_user_registr_dto.email.clone();
            let password = create_user_registr_dto.password.clone();
            let final_date = create_user_registr_dto.final_date.clone();

            let res_user1_opt: Option<UserRegistr> =
                self.find_user_registr_by_nickname_or_email(Some(&nickname), Some(&email))?;
            if res_user1_opt.is_some() {
                return Err("\"User Registration\" already exists.".to_string());
            }

            let idx: i32 = self.user_registr_vec.len().try_into().unwrap();
            let new_id: i32 = USER_REGISTR_ID + idx;
            let nickname = create_user_registr_dto.nickname.clone();
            let email = create_user_registr_dto.email.clone();

            let user_registr_saved: UserRegistr = UserRegistrOrmApp::new_user_registr(
                new_id, &nickname, &email, &password, final_date,
            );

            Ok(user_registr_saved)
        }
        /// Delete an entity (user_registration).
        fn delete_user_registr(&self, id: i32) -> Result<usize, String> {
            let exist_user_registr_opt: Option<&UserRegistr> =
                self.user_registr_vec.iter().find(|user_registr| user_registr.id == id);

            if exist_user_registr_opt.is_none() {
                Ok(0)
            } else {
                Ok(1)
            }
        }
        /// Delete all entities (user_registration) with an inactive "final_date".
        fn delete_inactive_final_date(
            &self,
            duration_in_days: Option<u16>,
        ) -> Result<usize, String> {
            let now = Utc::now();
            let duration = duration_in_days.unwrap_or(DURATION_IN_DAYS.into());
            let start_day_time = now - Duration::days(duration.into());
            let end_day_time = now.clone();

            let result = self
                .user_registr_vec
                .iter()
                .filter(|user_registr| {
                    user_registr.final_date > start_day_time
                        && user_registr.final_date < end_day_time
                })
                .count();

            Ok(result)
        }
    }
}
