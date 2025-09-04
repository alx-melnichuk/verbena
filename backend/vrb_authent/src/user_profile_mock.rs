// user_profile_mock

use chrono::Utc;

use crate::user_models::Profile;

// * * UserProfileMock * *

pub struct UserProfileMock {}

impl UserProfileMock {
    pub fn get_avatar(_user_id: i32) -> Option<String> {
        None
    }
    pub fn get_descript(user_id: i32) -> Option<String> {
        Some(format!("descript_{}", user_id))
    }
    pub fn get_theme(user_id: i32) -> Option<String> {
        if user_id % 2 == 0 {
            Some("dark".to_owned())
        } else {
            Some("light".to_owned())
        }
    }
    pub fn get_locale(user_id: i32) -> Option<String> {
        if user_id % 2 == 0 {
            Some("default".to_owned())
        } else {
            Some("en-US".to_owned())
        }
    }
    pub fn profile(user_id: i32) -> Profile {
        let now = Utc::now();
        Profile {
            user_id,
            avatar: Self::get_avatar(user_id),
            descript: Self::get_descript(user_id),
            theme: Self::get_theme(user_id),
            locale: Self::get_locale(user_id),
            created_at: now.clone(),
            updated_at: now,
        }
    }
}
