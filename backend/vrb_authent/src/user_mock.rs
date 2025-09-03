use vrb_common::user_validations;
use vrb_dbase::enm_user_role::UserRole;

// * * UserMock * *

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
}
