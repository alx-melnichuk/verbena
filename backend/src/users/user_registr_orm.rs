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

    use chrono::Utc;
    use diesel::{self, prelude::*};

    use crate::dbase;
    use crate::schema;
    use crate::users::user_registr_models::{CreateUserRegistrDto, UserRegistr};

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
            // Run query using Diesel to find user by nickname or email and return it (where final_date > now).
            let result = schema::user_registration::table
                .filter(
                    schema::user_registration::dsl::final_date.gt(current_now).and(
                        schema::user_registration::dsl::nickname
                            .eq(nickname2)
                            .or(schema::user_registration::dsl::email.eq(email2)),
                    ),
                )
                .first::<UserRegistr>(&mut conn)
                .optional()
                .map_err(|e| format!("{DB_USER_REGISTR}: {}", e.to_string()))?;

            #[rustfmt::skip] // #
            let res = if result.is_some() { "result.is_some();" } else { "result.is_none()" };
            log::debug!("{res}"); // #

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
                .map_err(|e| format!("{DB_USER_REGISTR}: {}", e.to_string()))?;

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
                    .map_err(|e| format!("{DB_USER_REGISTR}: {}", e.to_string()))?;

            Ok(count)
        }
    }
}

#[cfg(feature = "mockdata")]
pub mod tests {
    use chrono::{DateTime, Utc};

    use crate::users::user_registr_models::{CreateUserRegistrDto, UserRegistr};

    use super::UserRegistrOrm;

    pub const USER_REGISTR_ID_1: i32 = 1201;
    pub const USER_REGISTR_ID_2: i32 = 1202;

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
            for user_reg in user_reg_vec.iter() {
                user_registr_vec.push(Self::new_user_registr(
                    user_reg.id,
                    &user_reg.nickname.to_string(),
                    &user_reg.email.to_string(),
                    &user_reg.password.to_string(),
                    user_reg.final_date,
                ));
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
            let current_now = Utc::now();

            let result: Option<UserRegistr> = self
                .user_registr_vec
                .iter()
                .find(|user_registr| {
                    user_registr.final_date > current_now
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
                self.find_user_registr_by_nickname_or_email(&nickname, &email)?;
            if res_user1_opt.is_some() {
                return Err("\"User Registration\" already exists.".to_string());
            }

            let new_id = USER_REGISTR_ID_2;

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
    }
}
