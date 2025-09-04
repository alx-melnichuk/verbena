use vrb_common::user_validations;
use vrb_dbase::enm_user_role::UserRole;

use crate::user_models::{Session, User};

// * * UserMock * *

pub const ADMIN: u8 = 0;
pub const USER: u8 = 1;

pub const USER1: usize = 0;
pub const USER2: usize = 1;
pub const USER3: usize = 2;
pub const USER4: usize = 3;

pub const USER1_ID: i32 = 1100;
pub const USER2_ID: i32 = 1101;
pub const USER3_ID: i32 = 1102;
pub const USER4_ID: i32 = 1103;

pub const USER1_NAME: &str = "oliver_taylor";
pub const USER2_NAME: &str = "robert_brown";
pub const USER3_NAME: &str = "mary_williams";
pub const USER4_NAME: &str = "ava_wilson";

pub const USER_IDS: [i32; 4] = [USER1_ID, USER2_ID, USER3_ID, USER4_ID];
pub const USER_NAMES: [&str; 4] = [USER1_NAME, USER2_NAME, USER3_NAME, USER4_NAME];

pub struct UserMock {}

impl UserMock {
    pub fn nickname_min() -> String {
        (0..(user_validations::NICKNAME_MIN - 1)).map(|_| 'a').collect()
    }
    pub fn nickname_max() -> String {
        (0..(user_validations::NICKNAME_MAX + 1)).map(|_| 'a').collect()
    }
    pub fn nickname_wrong() -> String {
        let nickname: String = (0..(user_validations::NICKNAME_MIN - 1)).map(|_| 'a').collect();
        format!("{}#", nickname)
    }
    pub fn email_min() -> String {
        let suffix = "@us".to_owned();
        let email_min: usize = user_validations::EMAIL_MIN.into();
        let email: String = (0..(email_min - 1 - suffix.len())).map(|_| 'a').collect();
        format!("{}{}", email, suffix)
    }
    pub fn email_max() -> String {
        let email_max: usize = user_validations::EMAIL_MAX.into();
        let prefix: String = (0..64).map(|_| 'a').collect();
        let domain = ".ua";
        let len = email_max - prefix.len() - domain.len() + 1;
        let suffix: String = (0..len).map(|_| 'a').collect();
        format!("{}@{}{}", prefix, suffix, domain)
    }
    pub fn email_wrong() -> String {
        let suffix = "@".to_owned();
        let email_min: usize = user_validations::EMAIL_MIN.into();
        let email: String = (0..(email_min - suffix.len())).map(|_| 'a').collect();
        format!("{}{}", email, suffix)
    }
    pub fn password_min() -> String {
        (0..(user_validations::PASSWORD_MIN - 1)).map(|_| 'a').collect()
    }
    pub fn password_max() -> String {
        (0..(user_validations::PASSWORD_MAX + 1)).map(|_| 'a').collect()
    }
    pub fn password_wrong() -> String {
        (0..(user_validations::PASSWORD_MIN)).map(|_| 'a').collect()
    }
    pub fn role_wrong() -> String {
        let role = UserRole::all_values().get(0).unwrap().to_string();
        role[0..(role.len() - 1)].to_string()
    }
    pub fn get_num_token(user_id: i32) -> i32 {
        40000 + user_id
    }
    pub fn users(roles: &[u8]) -> (Vec<User>, Vec<Session>) {
        let mut user_vec: Vec<User> = Vec::new();
        let mut session_vec: Vec<Session> = Vec::new();
        let user_ids = USER_IDS.clone();

        let len = if roles.len() > user_ids.len() { user_ids.len() } else { roles.len() };
        for index in 0..len {
            let user_id = user_ids.get(index).unwrap().clone();
            let nickname = USER_NAMES.get(index).unwrap().to_lowercase();
            #[rustfmt::skip]
            let role = if *(roles.get(index).unwrap()) == ADMIN { UserRole::Admin } else { UserRole::User };

            let user = User::new(user_id, &nickname, &format!("{}@gmail.com", nickname), "", role);
            user_vec.push(user);
            let num_token = if user_id == USER1_ID { Some(Self::get_num_token(user_id)) } else { None };
            session_vec.push(Session { user_id, num_token });
        }
        (user_vec, session_vec)
    }
}
