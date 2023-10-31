use crate::users::user_models::{CreateUserDto, ModifyUserDto, User};

pub trait UserOrm {
    /// Find for an entity (user) by id.
    fn find_user_by_id(&self, id: i32) -> Result<Option<User>, String>;
    /// Find for entity (user) by nickname.
    fn find_user_by_nickname(&self, nickname: &str) -> Result<Option<User>, String>;
    /// Find for an entity (user) by email.
    fn find_user_by_email(&self, email: &str) -> Result<Option<User>, String>;
    /// Find for an entity (user) by nickname or email.
    fn find_user_by_nickname_or_email(
        &self,
        nickname: &str,
        email: &str,
    ) -> Result<Option<User>, String>;
    /// Add a new entity (user).
    fn create_user(&self, create_user_dto: &CreateUserDto) -> Result<User, String>;
    /// Modify an entity (user).
    fn modify_user(&self, id: i32, modify_user_dto: ModifyUserDto) -> Result<Option<User>, String>;
    /// Delete an entity (user).
    fn delete_user(&self, id: i32) -> Result<usize, String>;
}

pub mod cfg {
    use crate::dbase::DbPool;

    #[cfg(not(feature = "mockdata"))]
    use super::inst::UserOrmApp;
    #[cfg(not(feature = "mockdata"))]
    pub fn get_user_orm_app(pool: DbPool) -> UserOrmApp {
        UserOrmApp::new(pool)
    }

    #[cfg(feature = "mockdata")]
    use super::tests::UserOrmApp;
    #[cfg(feature = "mockdata")]
    pub fn get_user_orm_app(_: DbPool) -> UserOrmApp {
        UserOrmApp::new()
    }
}

#[cfg(not(feature = "mockdata"))]
pub mod inst {

    use diesel::{self, prelude::*};

    use crate::dbase;
    use crate::schema;

    use super::*;

    pub const CONN_POOL: &str = "ConnectionPool";
    pub const DB_USER: &str = "Db_User";

    #[derive(Debug, Clone)]
    pub struct UserOrmApp {
        pub pool: dbase::DbPool,
    }

    impl UserOrmApp {
        pub fn new(pool: dbase::DbPool) -> Self {
            UserOrmApp { pool }
        }
        pub fn get_conn(&self) -> Result<dbase::DbPooledConnection, String> {
            (&self.pool).get().map_err(|e| format!("{CONN_POOL}: {}", e.to_string()))
        }
    }

    impl UserOrm for UserOrmApp {
        /// Find for an entity (user) by id.
        fn find_user_by_id(&self, id: i32) -> Result<Option<User>, String> {
            // Get a connection from the P2D2 pool.
            let mut conn = self.get_conn()?;
            // Run query using Diesel to find user by id and return it.
            let user_opt = schema::users::table
                .filter(schema::users::dsl::id.eq(id))
                .first::<User>(&mut conn)
                .optional()
                .map_err(|e| format!("{DB_USER}: {}", e.to_string()))?;

            Ok(user_opt)
        }
        /// Find for entity (user) by nickname.
        fn find_user_by_nickname(&self, nickname: &str) -> Result<Option<User>, String> {
            if nickname.len() == 0 {
                return Ok(None);
            }
            // Get a connection from the P2D2 pool.
            let mut conn = self.get_conn()?;
            let nickname2 = nickname.to_lowercase();
            // Run query using Diesel to find user by nickname and return it.
            let user_opt = schema::users::table
                .filter(schema::users::dsl::nickname.eq(nickname2))
                .first::<User>(&mut conn)
                .optional()
                .map_err(|e| format!("{DB_USER}: {}", e.to_string()))?;

            Ok(user_opt)
        }
        /// Find for an entity (user) by email.
        fn find_user_by_email(&self, email: &str) -> Result<Option<User>, String> {
            if email.len() == 0 {
                return Ok(None);
            }
            // Get a connection from the P2D2 pool.
            let mut conn = self.get_conn()?;
            let email2 = email.to_lowercase();
            // Run query using Diesel to find user by email and return it.
            let user_opt = schema::users::table
                .filter(schema::users::dsl::email.eq(email2))
                .first::<User>(&mut conn)
                .optional()
                .map_err(|e| format!("{DB_USER}: {}", e.to_string()))?;

            Ok(user_opt)
        }
        /// Find for an entity (user) by nickname or email.
        fn find_user_by_nickname_or_email(
            &self,
            nickname: &str,
            email: &str,
        ) -> Result<Option<User>, String> {
            if nickname.len() == 0 || email.len() == 0 {
                return Ok(None);
            }
            let nickname2 = nickname.to_lowercase();
            let email2 = email.to_lowercase();
            // Get a connection from the P2D2 pool.
            let mut conn = self.get_conn()?;
            // Run query using Diesel to find user by nickname or email and return it.
            let user_opt = schema::users::table
                .filter(schema::users::dsl::nickname.eq(nickname2))
                .or_filter(schema::users::dsl::email.eq(email2))
                .first::<User>(&mut conn)
                .optional()
                .map_err(|e| format!("{DB_USER}: {}", e.to_string()))?;

            Ok(user_opt)
        }
        /// Add a new entity (user).
        fn create_user(&self, create_user_dto: &CreateUserDto) -> Result<User, String> {
            let mut create_user_dto2 = create_user_dto.clone();
            create_user_dto2.nickname = create_user_dto2.nickname.to_lowercase();
            create_user_dto2.email = create_user_dto2.email.to_lowercase();

            // Get a connection from the P2D2 pool.
            let mut conn = self.get_conn()?;
            // Run query using Diesel to add a new user entry.
            let user: User = diesel::insert_into(schema::users::table)
                .values(create_user_dto2)
                .returning(User::as_returning())
                .get_result(&mut conn)
                .map_err(|e| format!("{DB_USER}: {}", e.to_string()))?;

            Ok(user)
        }
        /// Modify an entity (user).
        fn modify_user(
            &self,
            id: i32,
            modify_user_dto: ModifyUserDto,
        ) -> Result<Option<User>, String> {
            let mut modify_user_dto2: ModifyUserDto = modify_user_dto.clone();

            if let Some(nickname) = modify_user_dto2.nickname {
                modify_user_dto2.nickname = Some(nickname.to_lowercase());
            }
            if let Some(email) = modify_user_dto2.email {
                modify_user_dto2.email = Some(email.to_lowercase());
            }
            // Get a connection from the P2D2 pool.
            let mut conn = self.get_conn()?;
            // Run query using Diesel to full or partially modify the user entry.
            let result = diesel::update(schema::users::dsl::users.find(id))
                .set(&modify_user_dto2)
                .returning(User::as_returning())
                .get_result(&mut conn)
                .optional()
                .map_err(|e| format!("{DB_USER}: {}", e.to_string()))?;

            Ok(result)
        }
        /// Delete an entity (user).
        fn delete_user(&self, id: i32) -> Result<usize, String> {
            // Get a connection from the P2D2 pool.
            let mut conn = self.get_conn()?;
            // Run query using Diesel to delete a entry (user).
            let count: usize = diesel::delete(schema::users::dsl::users.find(id))
                .execute(&mut conn)
                .map_err(|e| format!("{DB_USER}: {}", e.to_string()))?;

            Ok(count)
        }
    }
}

#[cfg(feature = "mockdata")]
pub mod tests {
    use chrono::{Duration, Utc};

    use crate::users::user_models::UserRole;

    use super::*;

    pub const USER_ID_1: i32 = 1101;
    pub const USER_ID_2: i32 = 1102;

    #[derive(Debug, Clone)]
    pub struct UserOrmApp {
        user_vec: Vec<User>,
    }

    impl UserOrmApp {
        /// Create a new instance.
        pub fn new() -> Self {
            UserOrmApp {
                user_vec: Vec::new(),
            }
        }
        /// Create a new instance with the specified user list.
        #[cfg(test)]
        pub fn create(user_list: Vec<User>) -> Self {
            let mut user_vec: Vec<User> = Vec::new();
            for user in user_list.iter() {
                user_vec.push(User {
                    id: user.id,
                    nickname: user.nickname.to_lowercase(),
                    email: user.email.to_lowercase(),
                    password: user.password.to_string(),
                    created_at: user.created_at,
                    updated_at: user.updated_at,
                    role: user.role,
                });
            }
            UserOrmApp { user_vec }
        }
        /// Create a new entity instance.
        pub fn new_user(id: i32, nickname: &str, email: &str, password: &str) -> User {
            let today = Utc::now();
            let cr_dt = today + Duration::minutes(-10);

            User {
                id,
                nickname: nickname.to_lowercase(),
                email: email.to_lowercase(),
                password: password.to_string(),
                created_at: cr_dt,
                updated_at: cr_dt,
                role: UserRole::User,
            }
        }
    }

    impl UserOrm for UserOrmApp {
        /// Find for an entity (user) by id.
        fn find_user_by_id(&self, id: i32) -> Result<Option<User>, String> {
            let result = self.user_vec.iter().find(|user| user.id == id).map(|user| user.clone());
            Ok(result)
        }
        /// Find for entity (user) by nickname.
        fn find_user_by_nickname(&self, nickname: &str) -> Result<Option<User>, String> {
            if nickname.len() == 0 {
                return Ok(None);
            }
            let nickname2 = nickname.to_lowercase();

            let result = self
                .user_vec
                .iter()
                .find(|user| user.nickname == nickname2)
                .map(|user| user.clone());

            Ok(result)
        }
        /// Find for an entity (user) by email.
        fn find_user_by_email(&self, email: &str) -> Result<Option<User>, String> {
            if email.len() == 0 {
                return Ok(None);
            }
            let email2 = email.to_lowercase();

            let result =
                self.user_vec.iter().find(|user| user.email == email2).map(|user| user.clone());

            Ok(result)
        }
        /// Find for an entity (user) by nickname or email.
        fn find_user_by_nickname_or_email(
            &self,
            nickname: &str,
            email: &str,
        ) -> Result<Option<User>, String> {
            if nickname.len() == 0 || email.len() == 0 {
                return Ok(None);
            }
            let nickname2 = nickname.to_lowercase();
            let email2 = email.to_lowercase();

            let result = self
                .user_vec
                .iter()
                .find(|user| user.nickname == nickname2 || user.email == email2)
                .map(|user| user.clone());

            Ok(result)
        }
        /// Add a new entity (user).
        fn create_user(&self, create_user_dto: &CreateUserDto) -> Result<User, String> {
            let nickname = &create_user_dto.nickname.to_lowercase();
            let email = &create_user_dto.email.to_lowercase();

            let user1_opt = self.find_user_by_nickname_or_email(nickname, email)?;
            if user1_opt.is_some() {
                return Err("Session already exists".to_string());
            }
            let password = &create_user_dto.password.clone();

            let user_saved: User = UserOrmApp::new_user(USER_ID_2, nickname, email, password);

            Ok(user_saved)
        }
        /// Modify an entity (user).
        fn modify_user(
            &self,
            id: i32,
            modify_user_dto: ModifyUserDto,
        ) -> Result<Option<User>, String> {
            let user_opt = self.user_vec.iter().find(|user| user.id == id);
            let user = match user_opt {
                Some(v) => v.clone(),
                None => {
                    return Ok(None);
                }
            };

            let nickname = modify_user_dto.nickname.unwrap_or(user.nickname.clone());
            let email = modify_user_dto.email.unwrap_or(user.email.clone());
            let password = modify_user_dto.password.unwrap_or(user.password.clone());
            let role = modify_user_dto.role.unwrap_or(user.role.clone());

            let mut user_saved: User = UserOrmApp::new_user(id, &nickname, &email, &password);
            user_saved.role = role;
            user_saved.created_at = user.created_at;
            user_saved.updated_at = Utc::now();

            Ok(Some(user_saved))
        }
        /// Delete an entity (user).
        fn delete_user(&self, id: i32) -> Result<usize, String> {
            let user_opt = self.user_vec.iter().find(|user| user.id == id);

            if user_opt.is_none() {
                Ok(0)
            } else {
                Ok(1)
            }
        }
    }
}
