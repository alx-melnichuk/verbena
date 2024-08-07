use crate::users::user_models::{ModifyUserDto, User};

pub trait UserOrm {
    /// Find for an entity (user) by nickname or email.
    fn find_user_by_nickname_or_email(
        &self,
        nickname: Option<&str>,
        email: Option<&str>,
    ) -> Result<Option<User>, String>;
    /// Modify an entity (user).
    fn modify_user(&self, id: i32, modify_user_dto: ModifyUserDto) -> Result<Option<User>, String>;
}

pub mod cfg {
    use crate::dbase::DbPool;

    #[cfg(not(feature = "mockdata"))]
    use super::impls::UserOrmApp;
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
pub mod impls {

    use diesel::{self, prelude::*};

    use crate::dbase;
    use crate::schema;

    use super::*;

    pub const CONN_POOL: &str = "ConnectionPool";

    #[derive(Debug, Clone)]
    pub struct UserOrmApp {
        pub pool: dbase::DbPool,
    }

    impl UserOrmApp {
        pub fn new(pool: dbase::DbPool) -> Self {
            UserOrmApp { pool }
        }
        pub fn get_conn(&self) -> Result<dbase::DbPooledConnection, String> {
            (&self.pool).get().map_err(|e| format!("{}: {}", CONN_POOL, e.to_string()))
        }
    }

    impl UserOrm for UserOrmApp {
        /// Find for an entity (user) by nickname or email.
        fn find_user_by_nickname_or_email(
            &self,
            nickname: Option<&str>,
            email: Option<&str>,
        ) -> Result<Option<User>, String> {
            let nickname2 = nickname.unwrap_or(&"".to_string()).to_lowercase(); // #?
            let nickname2_len = nickname2.len();
            let email2 = email.unwrap_or(&"".to_string()).to_lowercase();
            let email2_len = email2.len();

            if nickname2_len == 0 && email2_len == 0 {
                return Ok(None);
            }

            // Get a connection from the P2D2 pool.
            let mut conn = self.get_conn()?;

            // use diesel::sql_types::VarChar;
            // sql_function! { fn lower(a: VarChar) -> VarChar; }

            // Run query using Diesel to find user by nickname and return it.
            let sql_query_nickname = schema::users::table
                // .filter(lower(schema::users::dsl::nickname).eq(nickname2))
                .filter(schema::users::dsl::nickname.eq(nickname2))
                .select(schema::users::all_columns)
                .limit(1);
            // Run query using Diesel to find user by email and return it.
            let sql_query_email = schema::users::table
                .filter(schema::users::dsl::email.eq(email2))
                .select(schema::users::all_columns)
                .limit(1);

            let mut result_vec: Vec<User> = vec![];
            let table = "find_user_by_nickname_or_email";

            if nickname2_len > 0 && email2_len == 0 {
                // eprintln!("#sql_nick: `{}`", debug_query::<Pg, _>(&sql_query_nickname).to_string());
                let result_nickname_vec: Vec<User> = sql_query_nickname
                    .load(&mut conn)
                    .map_err(|e| format!("{}: {}", table, e.to_string()))?;
                result_vec.extend(result_nickname_vec);
            } else if nickname2_len == 0 && email2_len > 0 {
                // eprintln!("#sql_email: `{}`", debug_query::<Pg, _>(&sql_query_email).to_string());
                let result_email_vec: Vec<User> = sql_query_email
                    .load(&mut conn)
                    .map_err(|e| format!("{}: {}", table, e.to_string()))?;
                result_vec.extend(result_email_vec);
            } else {
                // This design (union two queries) allows the use of two separate indexes.
                let sql_query = sql_query_nickname.union_all(sql_query_email);
                // eprintln!("#sql_query: `{}`", debug_query::<Pg, _>(&sql_query).to_string());
                // Run query using Diesel to find user by nickname or email and return it.
                let result_nickname_email_vec: Vec<User> =
                    sql_query.load(&mut conn).map_err(|e| format!("{}: {}", table, e.to_string()))?;
                result_vec.extend(result_nickname_email_vec);
            }

            let result = if result_vec.len() > 0 {
                Some(result_vec[0].clone())
            } else {
                None
            };
            Ok(result)
        }
        /// Modify an entity (user).
        fn modify_user(&self, id: i32, modify_user_dto: ModifyUserDto) -> Result<Option<User>, String> {
            let mut modify_user_dto2: ModifyUserDto = modify_user_dto.clone();

            if let Some(nickname) = modify_user_dto2.nickname {
                modify_user_dto2.nickname = Some(nickname.to_lowercase()); // #?
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
                .map_err(|e| format!("modify_user: {}", e.to_string()))?;

            Ok(result)
        }
    }
}

#[cfg(feature = "mockdata")]
pub mod tests {
    use chrono::{Duration, Utc};

    use crate::users::user_models::UserRole;

    use super::*;

    pub const USER_ID: i32 = 1100;

    #[derive(Debug, Clone)]
    pub struct UserOrmApp {
        pub user_vec: Vec<User>,
    }

    impl UserOrmApp {
        /// Create a new instance.
        pub fn new() -> Self {
            UserOrmApp { user_vec: Vec::new() }
        }
        /// Create a new instance with the specified user list.
        #[cfg(test)]
        pub fn create(user_list: &[User]) -> Self {
            let mut user_vec: Vec<User> = Vec::new();
            for (idx, user) in user_list.iter().enumerate() {
                let delta: i32 = idx.try_into().unwrap();
                user_vec.push(User {
                    id: USER_ID + delta,
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
            let now = Utc::now();
            let cr_dt = now + Duration::minutes(-10);

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
        /// Find for an entity (user) by nickname or email.
        fn find_user_by_nickname_or_email(
            &self,
            nickname: Option<&str>,
            email: Option<&str>,
        ) -> Result<Option<User>, String> {
            let nickname2 = nickname.unwrap_or(&"".to_string()).to_lowercase();
            let nickname2_len = nickname2.len();
            let email2 = email.unwrap_or(&"".to_string()).to_lowercase();
            let email2_len = email2.len();

            if nickname2_len == 0 && email2_len == 0 {
                return Ok(None);
            }

            let result = self
                .user_vec
                .iter()
                .find(|user| {
                    (nickname2_len > 0 && user.nickname == nickname2) || (email2_len > 0 && user.email == email2)
                })
                .map(|user| user.clone());

            Ok(result)
        }
        /// Modify an entity (user).
        fn modify_user(&self, id: i32, modify_user_dto: ModifyUserDto) -> Result<Option<User>, String> {
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
    }
}
