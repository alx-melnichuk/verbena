use chrono::Utc;
#[cfg(not(feature = "mockdata"))]
use diesel::{self, prelude::*};

#[cfg(not(feature = "mockdata"))]
use crate::dbase;
#[cfg(not(feature = "mockdata"))]
use crate::schema;
use crate::utils::err;

use super::user_registr_models::{CreateUserRegistrDto, UserRegistr};

pub trait UserRegistrOrm {
    /// Find for an entity (user_registration) by nickname or email.
    fn find_user_registr_by_nickname_or_email(
        &self,
        nickname: &str,
        email: &str,
    ) -> Result<Option<UserRegistr>, String>;
    /// Add a new entity (user_registration).
    fn create_user_registr(
        &self,
        create_user_registr_dto: &CreateUserRegistrDto,
    ) -> Result<UserRegistr, String>;
    /// Delete an entity (user_registration).
    fn delete_user_registr(&self, id: i32) -> Result<usize, String>;
}

#[cfg(not(feature = "mockdata"))]
#[derive(Debug, Clone)]
pub struct UserRegistrOrmApp {
    pub pool: dbase::DbPool,
}

#[cfg(not(feature = "mockdata"))]
impl UserRegistrOrmApp {
    pub fn new(pool: dbase::DbPool) -> Self {
        UserRegistrOrmApp { pool }
    }
    pub fn get_conn(&self) -> Result<dbase::DbPooledConnection, String> {
        (&self.pool).get().map_err(|e| format!("ConnectionPool: {}", e.to_string()))
    }
}

#[cfg(not(feature = "mockdata"))]
impl UserRegistrOrm for UserRegistrOrmApp {
    /// Find for an entity (user_registration) by nickname or email.
    fn find_user_registr_by_nickname_or_email(
        &self,
        nickname: &str,
        email: &str,
    ) -> Result<Option<UserRegistr>, String> {
        if nickname.len() == 0 || email.len() == 0 {
            log::debug!("nickname or email are empty.");
            return Ok(None);
        }
        let nickname2 = nickname.to_lowercase();
        let email2 = email.to_lowercase();
        // Get a connection from the P2D2 pool.
        let mut conn = self.get_conn()?;
        let current_now = Utc::now();
        // Run query using Diesel to find user by nickname or email and return it.
        let result = schema::user_registration::table
            .filter(
                schema::user_registration::dsl::final_date.gt(current_now).and(
                    schema::user_registration::dsl::nickname
                        .eq(nickname2)
                        .or(schema::user_registration::dsl::email.eq(email2)),
                ),
            )
            // .filter(schema::user_registration::dsl::nickname.eq(nickname2))
            // .or_filter(schema::user_registration::dsl::email.eq(email2))
            // ?? Added "where final_date > now"
            .first::<UserRegistr>(&mut conn)
            .optional()
            .map_err(|e| {
                log::debug!("{}: {}", err::CD_DATABASE, e.to_string());
                format!("{}: {}", err::CD_DATABASE, e.to_string())
            })?;

        #[rustfmt::skip] // #
        let res = if result.is_some() { "result.is_some();" } else { "result.is_none()" };
        log::debug!("{res}");

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
        let password = create_user_registr_dto.password.clone();

        // Get a connection from the P2D2 pool.
        let mut conn = self.get_conn()?;
        // Run query using Diesel to add a new user entry.
        let user_registr: UserRegistr = diesel::insert_into(schema::user_registration::table)
            .values(create_user_registr_dto2)
            .returning(UserRegistr::as_returning())
            .get_result(&mut conn)
            .map_err(|e| {
                log::debug!("{}: {}", err::CD_DATABASE, e.to_string());
                format!("{}: {}", err::CD_DATABASE, e.to_string())
            })?;

        Ok(user_registr)
    }

    /// Delete an entity (user_registration).
    fn delete_user_registr(&self, id: i32) -> Result<usize, String> {
        // Get a connection from the P2D2 pool.
        let mut conn = self.get_conn()?;
        // Run query using Diesel to delete a entry (user_registration).
        let count: usize =
            diesel::delete(schema::user_registration::dsl::user_registration.find(id))
                .execute(&mut conn)
                .map_err(|e| {
                    log::debug!("{}: {}", err::CD_DATABASE, e.to_string());
                    format!("{}: {}", err::CD_DATABASE, e.to_string())
                })?;

        Ok(count)
    }
}

#[cfg(feature = "mockdata")]
pub mod tests {
    use chrono::{Duration, Utc};

    use super::user_registr_models::{CreateUserRegistrDto, UserRegistr};
    use super::*;

    use crate::sessions::hash_tools;

    #[derive(Debug, Clone)]
    pub struct UserRegistrOrmApp {
        user_registr_list: Vec<UserRegistr>,
    }

    impl UserRegistrOrmApp {
        /// Create a new instance.
        pub fn new() -> Self {
            UserRegistrOrmApp {
                user_registr_list: Vec::new(),
            }
        }
        /// Create a new instance with the specified user registr list.
        #[cfg(test)]
        pub fn create_user_registr(user_reg_list: Vec<UserRegistr>) -> Self {
            let mut user_registr_list: Vec<UserRegistr> = Vec::new();
            for user_reg in user_reg_list.iter() {
                user_registr_list.push(UserRegistr {
                    id: user_reg.id,
                    nickname: user_reg.nickname.to_lowercase(),
                    email: user_reg.email.to_lowercase(),
                    password: user_reg.password.to_string(),
                    final_date: user_reg.final_date,
                });
            }
            UserRegistrOrmApp { user_registr_list }
        }
        /// Create a new entity instance.
        pub fn new_user_registr(
            id: i32,
            nickname: &str,
            email: &str,
            password: &str,
            final_date: DateTime<Utc>,
        ) -> UserRegistr {
            let today = Utc::now();
            let cr_dt = today + Duration::seconds(-10);

            let password_hashed = hash_tools::hash(password).expect("Hashing error!");

            UserRegistr {
                id,
                nickname: nickname.to_lowercase(),
                email: email.to_lowercase(),
                password: password_hashed,
                final_date: final_date,
            }
        }
    }

    impl UserRegistrOrm for UserRegistrOrmApp {
        /// Find for an entity (user_registration) by nickname or email.
        fn find_user_registr_by_nickname_or_email(
            &self,
            nickname: &str,
            email: &str,
        ) -> Result<Option<UserRegistr>, String> {
            if nickname.len() == 0 || email.len() == 0 {
                return Ok(None);
            }
            let nickname2 = nickname.to_lowercase();
            let email2 = email.to_lowercase();

            let result: Option<UserRegistr> = self
                .user_registr_list
                .iter()
                .find(|user_registr| {
                    user_registr.nickname == nickname2 || user_registr.email == email2
                })
                .map(|user_registr| user_registr.clone());

            Ok(result)
        }

        /// Delete an entity (user_registration).
        fn delete_user_registr(&self, id: i32) -> Result<usize, String> {
            let exist_user_registr_opt: Option<&UserRegistr> =
                self.user_registr_list.iter().find(|user_registr| user_registr.id == id);

            if exist_user_registr_opt.is_none() {
                Ok(0)
            } else {
                Ok(1)
            }
        }
    }
}
