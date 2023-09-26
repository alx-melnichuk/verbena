use std::fmt;

use crate::users::user_models;

pub const LIMIT_DEFAULT: u8 = 10;
pub const LIMIT_MAX: u8 = 100;

#[derive(Debug, Clone, PartialEq)]
pub enum UserOrmError {
    #[cfg(not(feature = "mockdata"))]
    /// Error getting a database connection from the pool.
    ConnectionPool(String),
    #[cfg(not(feature = "mockdata"))]
    /// Error executing a query in the database.
    DataBase(String),
    /// Error while generating password hash.
    HashingPassword(String),
    /// A user with the given nickname or email already exists.
    UserAlreadyExists,
}

impl Into<String> for UserOrmError {
    fn into(self) -> String {
        self.to_string()
    }
}

impl std::error::Error for UserOrmError {}

impl fmt::Display for UserOrmError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            #[cfg(not(feature = "mockdata"))]
            UserOrmError::ConnectionPool(info) => {
                write!(f, "Error ConnectionPool: {}", info)
            }
            #[cfg(not(feature = "mockdata"))]
            UserOrmError::DataBase(info) => {
                write!(f, "Error DataBase: {}", info)
            }
            UserOrmError::HashingPassword(info) => {
                write!(f, "Error HashingPassword: {}", info)
            }
            UserOrmError::UserAlreadyExists => {
                write!(f, "Error, the given nickname or email already exists.")
            }
        }
    }
}

pub trait UserOrm {
    /// Run query using Diesel to find user by id and return it.
    fn find_user_by_id(&self, id: i32) -> Result<Option<user_models::User>, UserOrmError>;

    /// Run query using Diesel to find user by nickname and return it.
    fn find_user_by_nickname(
        &self,
        nickname: &str,
    ) -> Result<Option<user_models::User>, UserOrmError>;

    /// Run query using Diesel to find user by email and return it.
    fn find_user_by_email(&self, email: &str) -> Result<Option<user_models::User>, UserOrmError>;

    /// Run query using Diesel to find user by nickname or email and return it.
    fn find_user_by_nickname_or_email(
        &self,
        nickname_or_email: &str,
    ) -> Result<Option<user_models::User>, UserOrmError>;

    /// Run query using Diesel to add a new user entry.
    fn create_user(
        &self,
        create_user_dto: &user_models::CreateUserDto,
    ) -> Result<user_models::User, UserOrmError>;

    /// Run query using Diesel to full or partially modify the user entry.
    fn modify_user(
        &self,
        id: i32,
        new_user_dto: user_models::ModifyUserDto,
    ) -> Result<Option<user_models::User>, UserOrmError>;

    /// Run query using Diesel to delete a user entry.
    fn delete_user(&self, id: i32) -> Result<usize, UserOrmError>;
}

#[cfg(not(feature = "mockdata"))]
use diesel::prelude::*;

#[cfg(not(feature = "mockdata"))]
use crate::dbase;
#[cfg(not(feature = "mockdata"))]
use crate::schema;
#[cfg(not(feature = "mockdata"))]
use crate::sessions::hash_tools;

#[cfg(not(feature = "mockdata"))]
#[derive(Debug, Clone)]
pub struct UserOrmApp {
    pub pool: dbase::DbPool,
}

#[cfg(not(feature = "mockdata"))]
impl UserOrmApp {
    pub fn new(pool: dbase::DbPool) -> Self {
        UserOrmApp { pool }
    }
    pub fn get_conn(&self) -> Result<dbase::DbPooledConnection, UserOrmError> {
        (&self.pool).get().map_err(|e| UserOrmError::ConnectionPool(e.to_string()))
    }
}

#[cfg(not(feature = "mockdata"))]
impl UserOrm for UserOrmApp {
    fn find_user_by_id(&self, id: i32) -> Result<Option<user_models::User>, UserOrmError> {
        // Get a connection from the P2D2 pool.
        let mut conn = self.get_conn()?;

        // Run query using Diesel to find user by id.
        let opt_user_dto: Option<user_models::User> = schema::users::table
            .filter(schema::users::dsl::id.eq(id))
            .first::<user_models::User>(&mut conn)
            .optional()
            .map_err(|e| {
                log::warn!("UsOrmError::DataBase: {}", e.to_string());
                UserOrmError::DataBase(e.to_string())
            })?;

        Ok(opt_user_dto)
    }

    fn find_user_by_nickname(
        &self,
        nickname: &str,
    ) -> Result<Option<user_models::User>, UserOrmError> {
        // Get a connection from the P2D2 pool.
        let mut conn = self.get_conn()?;

        if nickname.len() == 0 {
            return Ok(None);
        }
        let nickname2 = nickname.to_lowercase();

        let opt_user_dto: Option<user_models::User> = schema::users::table
            .filter(schema::users::dsl::nickname.eq(nickname2))
            .first::<user_models::User>(&mut conn)
            .optional()
            .map_err(|e| {
                log::warn!("UsOrmError::DataBase: {}", e.to_string());
                UserOrmError::DataBase(e.to_string())
            })?;

        Ok(opt_user_dto)
    }

    fn find_user_by_email(&self, email: &str) -> Result<Option<user_models::User>, UserOrmError> {
        // Get a connection from the P2D2 pool.
        let mut conn = self.get_conn()?;

        if email.len() == 0 {
            return Ok(None);
        }
        let email2 = email.to_lowercase();

        let opt_user_dto: Option<user_models::User> = schema::users::table
            .filter(schema::users::dsl::email.eq(email2))
            .first::<user_models::User>(&mut conn)
            .optional()
            .map_err(|e| {
                log::warn!("UsOrmError::DataBase: {}", e.to_string());
                UserOrmError::DataBase(e.to_string())
            })?;

        Ok(opt_user_dto)
    }

    fn find_user_by_nickname_or_email(
        &self,
        nickname_or_email: &str,
    ) -> Result<Option<user_models::User>, UserOrmError> {
        // Get a connection from the P2D2 pool.
        let mut conn = self.get_conn()?;

        if nickname_or_email.len() == 0 {
            return Ok(None);
        }
        let nickname2 = nickname_or_email.to_lowercase();
        let email2 = nickname2.clone();

        let opt_user_dto: Option<user_models::User> = schema::users::table
            .filter(schema::users::dsl::nickname.eq(nickname2))
            .or_filter(schema::users::dsl::email.eq(email2))
            .first::<user_models::User>(&mut conn)
            .optional()
            .map_err(|e| {
                log::warn!("UsOrmError::DataBase: {}", e.to_string());
                UserOrmError::DataBase(e.to_string())
            })?;

        Ok(opt_user_dto)
    }

    fn create_user(
        &self,
        create_user_dto: &user_models::CreateUserDto,
    ) -> Result<user_models::User, UserOrmError> {
        // #? Checking data validity.

        // Get a connection from the P2D2 pool.
        let mut conn = self.get_conn()?;

        let mut create_user_dto2 = create_user_dto.clone();
        create_user_dto2.nickname = create_user_dto2.nickname.to_lowercase();
        create_user_dto2.email = create_user_dto2.email.to_lowercase();

        let password_hashed = hash_tools::hash(&create_user_dto.password.clone())
            .map_err(|e| UserOrmError::HashingPassword(e.to_string()))?;
        create_user_dto2.password = password_hashed;

        let user: user_models::User = diesel::insert_into(schema::users::table)
            .values(create_user_dto2)
            .returning(user_models::User::as_returning())
            .get_result(&mut conn)
            .map_err(|e| {
                log::warn!("UsOrmError::DataBase: {}", e.to_string());
                UserOrmError::DataBase(e.to_string())
            })?;

        Ok(user)
    }

    fn modify_user(
        &self,
        id: i32,
        modify_user_dto: user_models::ModifyUserDto,
    ) -> Result<Option<user_models::User>, UserOrmError> {
        // #? Checking data validity.

        let res_user: Option<user_models::User> = self.find_user_by_id(id)?; // #?
        if res_user.is_none() {
            return Ok(None);
        }

        // Get a connection from the P2D2 pool.
        let mut conn = self.get_conn()?;

        let mut modify_user_dto2: user_models::ModifyUserDto = modify_user_dto.clone();

        if let Some(nickname_val) = modify_user_dto2.nickname {
            modify_user_dto2.nickname = Some(nickname_val.to_lowercase());
        }
        if let Some(email_val) = modify_user_dto2.email {
            modify_user_dto2.email = Some(email_val.to_lowercase());
        }
        if let Some(password_val) = modify_user_dto2.password {
            let password_hashed = hash_tools::hash(&password_val)
                .map_err(|e| UserOrmError::HashingPassword(e.to_string()))?;
            modify_user_dto2.password = Some(password_hashed);
        }
        // if let Some(role_val) = modify_user_dto2.role {
        //     // #!! check current role is admin
        // }

        let result = diesel::update(schema::users::dsl::users.find(id))
            .set(&modify_user_dto2)
            .returning(user_models::User::as_returning())
            .get_result(&mut conn)
            .optional()
            .map_err(|e| {
                log::warn!("UsOrmError::DataBase: {}", e.to_string());
                UserOrmError::DataBase(e.to_string())
            })?;

        Ok(result)
    }
    fn delete_user(&self, id: i32) -> Result<usize, UserOrmError> {
        // Get a connection from the P2D2 pool.
        let mut conn = self.get_conn()?;

        let count: usize = diesel::delete(schema::users::dsl::users.find(id))
            .execute(&mut conn)
            .map_err(|e| {
                log::warn!("UsOrmError::DataBase: {}", e.to_string());
                UserOrmError::DataBase(e.to_string())
            })?;

        Ok(count)
    }
}
