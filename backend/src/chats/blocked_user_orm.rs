use crate::chats::blocked_user_models::{BlockedUser, CreateBlockedUser, DeleteBlockedUser};

pub trait BlockedUserOrm {
    /// Add a new entry (blocked_user).
    fn create_blocked_user(&self, create_blocked_user: CreateBlockedUser) -> Result<Option<BlockedUser>, String>;
    /// Delete an entity (blocked_user).
    fn delete_blocked_user(&self, delete_blocked_user: DeleteBlockedUser) -> Result<Option<BlockedUser>, String>;
}

pub mod cfg {
    use crate::dbase::DbPool;

    #[cfg(not(all(test, feature = "mockdata")))]
    use super::impls::BlockedUserOrmApp;
    #[cfg(not(all(test, feature = "mockdata")))]
    pub fn get_blocked_user_orm_app(pool: DbPool) -> BlockedUserOrmApp {
        BlockedUserOrmApp::new(pool)
    }

    #[cfg(all(test, feature = "mockdata"))]
    use super::tests::BlockedUserOrmApp;
    #[cfg(all(test, feature = "mockdata"))]
    pub fn get_blocked_user_orm_app(_: DbPool) -> BlockedUserOrmApp {
        BlockedUserOrmApp::new()
    }
}

#[cfg(not(all(test, feature = "mockdata")))]
pub mod impls {
    use std::time::Instant as tm;

    use diesel::{self, prelude::*, sql_types};
    use log::{info, log_enabled, Level::Info};

    use crate::chats::{
        blocked_user_models::{BlockedUser, CreateBlockedUser, DeleteBlockedUser},
        blocked_user_orm::BlockedUserOrm,
    };
    use crate::dbase;
    use crate::validators::Validator;

    pub const CONN_POOL: &str = "ConnectionPool";

    #[derive(Debug, Clone)]
    pub struct BlockedUserOrmApp {
        pub pool: dbase::DbPool,
    }

    impl BlockedUserOrmApp {
        pub fn new(pool: dbase::DbPool) -> Self {
            BlockedUserOrmApp { pool }
        }
        pub fn get_conn(&self) -> Result<dbase::DbPooledConnection, String> {
            (&self.pool).get().map_err(|e| format!("{}: {}", CONN_POOL, e.to_string()))
        }
    }

    impl BlockedUserOrm for BlockedUserOrmApp {
        /// Add a new entry (blocked_user).
        fn create_blocked_user(&self, create_blocked_user: CreateBlockedUser) -> Result<Option<BlockedUser>, String> {
            let timer = if log_enabled!(Info) { Some(tm::now()) } else { None };

            let validation_res = create_blocked_user.validate();
            if let Err(validation_errors) = validation_res {
                let buff: Vec<String> = validation_errors.into_iter().map(|v| v.message.to_string()).collect();
                return Err(buff.join("','"));
            }

            // Get a connection from the P2D2 pool.
            let mut conn = self.get_conn()?;

            let query = diesel::sql_query("select * from create_blocked_user($1,$2,$3);")
                .bind::<sql_types::Integer, _>(create_blocked_user.user_id) // $1
                .bind::<sql_types::Nullable<sql_types::Integer>, _>(create_blocked_user.blocked_id) // $2
                .bind::<sql_types::Nullable<sql_types::Text>, _>(create_blocked_user.blocked_nickname); // $3

            // Run a query with Diesel to create a new user and return it.
            let blocked_user = query
                .get_result::<BlockedUser>(&mut conn)
                .optional()
                .map_err(|e| format!("create_blocked_user: {}", e.to_string()))?;

            if let Some(timer) = timer {
                info!("create_blocked_user() time: {}", format!("{:.2?}", timer.elapsed()));
            }
            Ok(blocked_user)
        }

        /// Delete an entity (blocked_user).
        #[rustfmt::skip]
        fn delete_blocked_user(&self, delete_blocked_user: DeleteBlockedUser) -> Result<Option<BlockedUser>, String> {
            let timer = if log_enabled!(Info) { Some(tm::now()) } else { None };
            let user_id = delete_blocked_user.user_id;
            let blocked_id = delete_blocked_user.blocked_id.clone();
            let nickname = delete_blocked_user.blocked_nickname.clone();
            #[rustfmt::skip]
            eprintln!("delete_blocked_user() user_id: {}, blocked_id: {:?}, blocked_nickname: {:?}", user_id, blocked_id, nickname);
            let validation_res = delete_blocked_user.validate();
            if let Err(validation_errors) = validation_res {
                let buff: Vec<String> = validation_errors.into_iter().map(|v| v.message.to_string()).collect();
                return Err(buff.join("','"));
            }

            // Get a connection from the P2D2 pool.
            let mut conn = self.get_conn()?;

            let query = diesel::sql_query("select * from delete_blocked_user($1,$2,$3);")
                .bind::<sql_types::Integer, _>(delete_blocked_user.user_id) // $1
                .bind::<sql_types::Nullable<sql_types::Integer>, _>(delete_blocked_user.blocked_id) // $2
                .bind::<sql_types::Nullable<sql_types::Text>, _>(delete_blocked_user.blocked_nickname); // $3

            // Run a query with Diesel to delete the entity and return it.
            let blocked_user = query
                .get_result::<BlockedUser>(&mut conn)
                .optional()
                .map_err(|e| format!("delete_blocked_user: {}", e.to_string()))?;

            eprintln!("delete_blocked_user() res_blocked_user: {:?}", blocked_user);
            if let Some(timer) = timer {
                info!("delete_blocked_user() time: {}", format!("{:.2?}", timer.elapsed()));
            }
            Ok(blocked_user)

        }
    }
}

#[cfg(all(test, feature = "mockdata"))]
pub mod tests {
    use std::cell::RefCell;

    use chrono::Utc;

    use crate::chats::{
        blocked_user_models::{BlockedUser, CreateBlockedUser, DeleteBlockedUser},
        blocked_user_orm::BlockedUserOrm,
    };
    use crate::validators::Validator;

    pub const BLOCKED_USER_ID: i32 = 1700;

    #[derive(Debug, Clone)]
    pub struct UserMini {
        pub id: i32,
        pub name: String,
    }

    #[derive(Debug, Clone)]
    pub struct BlockedUserOrmApp {
        pub blocked_user_vec: Box<RefCell<Vec<BlockedUser>>>,
        pub user_vec: Vec<UserMini>,
    }

    impl BlockedUserOrmApp {
        /// Create a new instance.
        pub fn new() -> Self {
            BlockedUserOrmApp {
                blocked_user_vec: Box::new(RefCell::new(Vec::new())),
                user_vec: Vec::new(),
            }
        }
        /// Create a new instance with the specified BlockedUser list.
        pub fn create(blocked_user_list: &[BlockedUser], users_list: &[UserMini]) -> Self {
            let mut blocked_user_vec: Vec<BlockedUser> = Vec::new();
            let user_vec: Vec<UserMini> = Vec::from(users_list);
            for (idx, blocked_user) in blocked_user_list.iter().enumerate() {
                let delta: i32 = idx.try_into().unwrap();
                let id = BLOCKED_USER_ID + delta;
                let new_blocked_user = BlockedUser::new(
                    id,
                    blocked_user.user_id,
                    blocked_user.blocked_id,
                    blocked_user.blocked_nickname.clone(),
                    Some(blocked_user.block_date.clone()),
                );
                blocked_user_vec.push(new_blocked_user);
            }
            BlockedUserOrmApp {
                blocked_user_vec: Box::new(RefCell::new(blocked_user_vec)),
                user_vec,
            }
        }
        pub fn find_user_by_id(&self, id: i32) -> Option<UserMini> {
            self.user_vec.iter().find(|v| v.id == id).map(|v| v.clone())
        }
        pub fn find_user_by_name(&self, name: &str) -> Option<UserMini> {
            self.user_vec.iter().find(|v| v.name == name).map(|v| v.clone())
        }
    }

    impl BlockedUserOrm for BlockedUserOrmApp {
        /// Add a new entry (blocked_user).
        fn create_blocked_user(&self, create_blocked_user: CreateBlockedUser) -> Result<Option<BlockedUser>, String> {
            if create_blocked_user.blocked_id.is_none() && create_blocked_user.blocked_nickname.is_none() {
                return Ok(None);
            }
            let validation_res = create_blocked_user.validate();
            if let Err(validation_errors) = validation_res {
                let buff: Vec<String> = validation_errors.into_iter().map(|v| v.message.to_string()).collect();
                return Err(buff.join("','"));
            }
            let mut opt_user_mini: Option<UserMini> = None;
            if let Some(blocked_id) = create_blocked_user.blocked_id {
                opt_user_mini = self.find_user_by_id(blocked_id);
            } else if let Some(blocked_nickname) = create_blocked_user.blocked_nickname {
                opt_user_mini = self.find_user_by_name(&blocked_nickname);
            }
            let mut result: Option<BlockedUser> = None;
            let mut vec = (*self.blocked_user_vec).borrow_mut();
            // eprintln!("   @ create_blocked_user()           len: {}", vec.len()); // len: 3
            if let Some(user_mini) = opt_user_mini {
                let opt_blocked_user = vec
                    .iter()
                    .find(|v| {
                        (*v).user_id == create_blocked_user.user_id
                            && (*v).blocked_id == user_mini.id
                            && (*v).blocked_nickname.eq(&user_mini.name)
                    })
                    .map(|v| v.clone());

                if let Some(blocked_user) = opt_blocked_user {
                    result = Some(blocked_user);
                } else {
                    let cnt = vec.len();
                    let idx: i32 = cnt.try_into().unwrap();
                    let blocked_user = BlockedUser::new(
                        BLOCKED_USER_ID + idx,
                        create_blocked_user.user_id,
                        user_mini.id,
                        user_mini.name.clone(),
                        Some(Utc::now()),
                    );
                    vec.push(blocked_user.clone());
                    result = Some(blocked_user);
                    // eprintln!("   @ create_blocked_user() .push()   len: {}", vec.len()); // len: 4
                }
            }
            Ok(result)
        }

        /// Delete an entity (blocked_user).
        fn delete_blocked_user(&self, delete_blocked_user: DeleteBlockedUser) -> Result<Option<BlockedUser>, String> {
            if delete_blocked_user.blocked_id.is_none() && delete_blocked_user.blocked_nickname.is_none() {
                return Ok(None);
            }
            let validation_res = delete_blocked_user.validate();
            if let Err(validation_errors) = validation_res {
                let buff: Vec<String> = validation_errors.into_iter().map(|v| v.message.to_string()).collect();
                return Err(buff.join("','"));
            }
            let mut opt_user_mini: Option<UserMini> = None;
            if let Some(blocked_id) = delete_blocked_user.blocked_id {
                opt_user_mini = self.find_user_by_id(blocked_id);
            } else if let Some(blocked_nickname) = delete_blocked_user.blocked_nickname {
                opt_user_mini = self.find_user_by_name(&blocked_nickname);
            }

            let mut result: Option<BlockedUser> = None;
            let mut vec = (*self.blocked_user_vec).borrow_mut();
            // eprintln!("   @ delete_blocked_user()           len: {}", vec.len()); // len: 3
            if let Some(user_mini) = opt_user_mini {
                let opt_index = vec.iter().position(|v| {
                    (*v).user_id == delete_blocked_user.user_id
                        && (*v).blocked_id == user_mini.id
                        && (*v).blocked_nickname.eq(&user_mini.name)
                });
                if let Some(index) = opt_index {
                    let blocked_user = vec.remove(index);
                    result = Some(blocked_user);
                    // eprintln!("   @ delete_blocked_user() .remove() len: {}", vec.len()); // len: 2
                }
            }
            Ok(result)
        }
    }
}
