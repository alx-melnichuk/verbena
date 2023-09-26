// #[cfg(feature = "mockdata")]

use chrono::{Duration, Utc};

use crate::sessions::hash_tools;

use super::user_models;
use super::user_orm::{UserOrm, UserOrmError};

#[derive(Debug, Clone)]
pub struct UserOrmApp {
    users: Vec<user_models::User>,
}

impl UserOrmApp {
    /// Create a new instance.
    pub fn new() -> Self {
        let users: Vec<user_models::User> = UserOrmApp::create_users();
        UserOrmApp { users }
    }
    /// Create a new instance with the specified users.
    #[cfg(test)]
    pub fn create(user_list: Vec<user_models::User>) -> Self {
        let mut users: Vec<user_models::User> = Vec::new();
        for user in user_list.iter() {
            users.push(user_models::User {
                id: user.id,
                nickname: user.nickname.to_lowercase(),
                email: user.email.to_lowercase(),
                password: user.password.to_string(),
                created_at: user.created_at,
                updated_at: user.updated_at,
                role: user.role,
            });
        }
        UserOrmApp { users }
    }
    /// Create a new entity instance.
    pub fn new_user(
        id: i32,
        nickname: String,
        email: String,
        password: String,
    ) -> user_models::User {
        let today = Utc::now();
        let cr_dt = today + Duration::seconds(-10);

        let password_hashed = hash_tools::hash(&password).expect("Hashing error!");

        user_models::User {
            id,
            nickname: nickname.to_lowercase(),
            email: email.to_lowercase(),
            password: password_hashed,
            created_at: cr_dt,
            updated_at: cr_dt,
            role: user_models::UserRole::User,
        }
    }
    pub fn create_users() -> Vec<user_models::User> {
        let mut buff: Vec<user_models::User> = Vec::new();
        buff.push(Self::new_user(
            1,
            "James_Smith".to_string(),
            "James_Smith@gmail.com".to_string(),
            "password1234".to_string(),
        ));
        buff.push(Self::new_user(
            2,
            "Mary_Williams".to_string(),
            "Mary_Williams@gmail.com".to_string(),
            "123justgetit".to_string(),
        ));
        buff.push(Self::new_user(
            3,
            "Robert_Brown".to_string(),
            "Robert_Brown@gmail.com".to_string(),
            "mostsecurepass".to_string(),
        ));
        buff.push(Self::new_user(
            4,
            "Linda_Miller".to_string(),
            "Linda_Miller@gmail.com".to_string(),
            "mostsecurepass".to_string(),
        ));

        buff
    }
}

impl UserOrm for UserOrmApp {
    fn find_user_by_id(&self, id: i32) -> Result<Option<user_models::User>, UserOrmError> {
        let result: Option<user_models::User> =
            self.users.iter().find(|user| user.id == id).map(|user| user.clone());
        Ok(result)
    }

    fn find_user_by_nickname(
        &self,
        nickname: &str,
    ) -> Result<Option<user_models::User>, UserOrmError> {
        let nickname2 = nickname.to_lowercase();

        let exist_user_opt: Option<user_models::User> = self
            .users
            .iter()
            .find(|user| user.nickname == nickname2)
            .map(|user| user.clone());

        Ok(exist_user_opt)
    }

    fn find_user_by_email(&self, email: &str) -> Result<Option<user_models::User>, UserOrmError> {
        let email2 = email.to_lowercase();

        let exist_user_opt: Option<user_models::User> =
            self.users.iter().find(|user| user.email == email2).map(|user| user.clone());

        Ok(exist_user_opt)
    }

    fn find_user_by_nickname_or_email(
        &self,
        nickname_or_email: &str,
    ) -> Result<Option<user_models::User>, UserOrmError> {
        let nickname = nickname_or_email.to_lowercase();
        let email = nickname.clone();

        let result: Option<user_models::User> = self
            .users
            .iter()
            .find(|user| user.nickname == nickname || user.email == email)
            .map(|user| user.clone());

        Ok(result)
    }

    fn create_user(
        &self,
        create_user_dto: &user_models::CreateUserDto,
    ) -> Result<user_models::User, UserOrmError> {
        // #? Checking data validity.

        let nickname = &create_user_dto.nickname.clone();
        let email = &create_user_dto.email.clone();

        let res_user1: Option<user_models::User> = self.find_user_by_nickname(nickname)?;

        if res_user1.is_some() {
            log::warn!("UsOrmError::UserAlreadyExists: nickname: {nickname}");
            return Err(UserOrmError::UserAlreadyExists);
        }

        let res_user2: Option<user_models::User> = self.find_user_by_email(email)?;

        if res_user2.is_some() {
            log::warn!("UsOrmError::UserAlreadyExists: email: {email}");
            return Err(UserOrmError::UserAlreadyExists);
        }

        let password = &create_user_dto.password.clone();

        let user_saved: user_models::User = UserOrmApp::new_user(
            1001,
            nickname.to_string(),
            email.to_string(),
            password.to_string(),
        );

        Ok(user_saved)
    }

    fn modify_user(
        &self,
        id: i32,
        modify_user_dto: user_models::ModifyUserDto,
    ) -> Result<Option<user_models::User>, UserOrmError> {
        // #? Checking data validity.

        let exist_user_opt: Option<&user_models::User> =
            self.users.iter().find(|user| user.id == id);

        if exist_user_opt.is_none() {
            return Ok(None);
        }
        let mut res_user = exist_user_opt.unwrap().clone();

        let modify_user_dto2 = modify_user_dto.clone();

        let nickname = modify_user_dto2.nickname.unwrap_or(res_user.nickname.clone());
        let email = modify_user_dto2.email.unwrap_or(res_user.email.clone());
        let password = modify_user_dto2.password.unwrap_or("".to_string());
        let password_len = password.len();

        let user_saved: user_models::User = UserOrmApp::new_user(id, nickname, email, password);

        res_user.nickname = user_saved.nickname.clone();
        res_user.email = user_saved.email.clone();
        res_user.password = if password_len > 0 {
            user_saved.password
        } else {
            res_user.password
        };
        res_user.role = if modify_user_dto2.role.is_some() {
            modify_user_dto2.role.unwrap()
        } else {
            res_user.role
        };
        res_user.updated_at = Utc::now();

        Ok(Some(res_user))
    }

    fn delete_user(&self, id: i32) -> Result<usize, UserOrmError> {
        let exist_user_opt: Option<&user_models::User> =
            self.users.iter().find(|user| user.id == id);

        if exist_user_opt.is_none() {
            Ok(0)
        } else {
            Ok(1)
        }
    }
}
