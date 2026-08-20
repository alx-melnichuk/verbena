use vrb_authent::user_models::Profile;
use vrb_db::db::DbPool2;

use crate::profile_models::{ModifyUserProfile, UserProfile};

pub trait ProfileDb {
    /// Get an entity (profile) by ID.
    fn get_profile_by_id(&self, user_id: i32) -> impl std::future::Future<Output = Result<Option<Profile>, String>> + Send;

    /// Get an entities  (user, profile) by ID.
    fn get_user_profile_by_id(&self, user_id: i32) -> impl std::future::Future<Output = Result<Option<UserProfile>, String>> + Send;

    /// Modify an entity (profile, user).
    fn modify_user_profile(
        &self,
        user_id: i32,
        modify_profile: ModifyUserProfile,
    ) -> impl std::future::Future<Output = Result<Option<UserProfile>, String>> + Send;

    /// Filter for the list of stream logos by user ID.
    fn filter_stream_logos(&self, user_id: i32) -> impl std::future::Future<Output = Result<Vec<String>, String>> + Send;
}

#[cfg(not(all(test, feature = "mockdata")))]
pub fn get_profile_db_app(db_pool: DbPool2) -> impls::ProfileDbApp {
    impls::ProfileDbApp::new(db_pool)
}
#[cfg(all(test, feature = "mockdata"))]
pub fn get_profile_db_app(_: DbPool2) -> tests::ProfileDbApp {
    tests::ProfileDbApp::new()
}

#[cfg(not(all(test, feature = "mockdata")))]
pub mod impls {
    use std::time::Instant as tm;

    use log::{Level::Info, info, log_enabled};
    use sqlx;
    use vrb_authent::user_models::Profile;
    use vrb_db::db::DbPool2;

    use crate::profile_db::ProfileDb;
    use crate::profile_models::{ModifyUserProfile, StreamLogo, UserProfile};

    pub const CONN_POOL: &str = "ConnectionPool";

    #[derive(Debug, Clone)]
    pub struct ProfileDbApp {
        pub db_pool: DbPool2,
    }

    impl ProfileDbApp {
        pub fn new(db_pool: DbPool2) -> Self {
            ProfileDbApp { db_pool }
        }
    }

    impl ProfileDb for ProfileDbApp {
        /// Get an entity (profile) by ID.
        async fn get_profile_by_id(&self, user_id: i32) -> Result<Option<Profile>, String> {
            let timer = if log_enabled!(Info) { Some(tm::now()) } else { None };

            let result: Option<Profile> = sqlx::query_as(
                "SELECT user_id, avatar, descript, theme, locale, created_at, updated_at \
                FROM profiles \
                WHERE user_id = $1 \
                LIMIT 1",
            )
            .bind(user_id)
            .fetch_optional(&self.db_pool)
            .await
            .map_err(|e| format!("get_profile_by_id: {}", e.to_string()))?;

            if let Some(timer) = timer {
                info!("get_profile_by_id() time: {}", format!("{:.2?}", timer.elapsed()));
            }
            Ok(result)
        }

        /// Get an entities (user, profile) by ID.
        async fn get_user_profile_by_id(&self, user_id: i32) -> Result<Option<UserProfile>, String> {
            let timer = if log_enabled!(Info) { Some(tm::now()) } else { None };

            let result: Option<UserProfile> = sqlx::query_as(
                "SELECT user_id, nickname, email, \"role\", avatar, descript, theme, locale, created_at, updated_at \
                FROM get_user_profile_by_id($1) \
                LIMIT 1",
            )
            .bind(user_id)
            .fetch_optional(&self.db_pool)
            .await
            .map_err(|e| format!("get_user_profile_by_id: {}", e.to_string()))?;

            if let Some(timer) = timer {
                info!("get_user_profile_by_id() time: {}", format!("{:.2?}", timer.elapsed()));
            }
            Ok(result)
        }

        /// Modify an entity (user, profile).
        async fn modify_user_profile(&self, user_id: i32, modify_profile: ModifyUserProfile) -> Result<Option<UserProfile>, String> {
            let timer = if log_enabled!(Info) { Some(tm::now()) } else { None };

            let nickname = modify_profile.nickname.map(|v| v.to_lowercase());
            let email = modify_profile.email.map(|v| v.to_lowercase());
            let avatar = match modify_profile.avatar {
                Some(value1) => match value1 {
                    Some(value2) => Some(value2),
                    None => Some("".to_string()),
                },
                None => None,
            };

            let result: Option<UserProfile> = sqlx::query_as(
                "SELECT user_id, nickname, email, \"role\", avatar, descript, theme, locale, created_at, updated_at \
                FROM modify_user_profile($1,$2,$3,$4,$5,$6,$7,$8,$9) \
                LIMIT 1",
            )
            .bind(user_id)
            .bind(nickname)
            .bind(email)
            .bind(modify_profile.password)
            .bind(modify_profile.role)
            .bind(avatar)
            .bind(modify_profile.descript)
            .bind(modify_profile.theme)
            .bind(modify_profile.locale)
            .fetch_optional(&self.db_pool)
            .await
            .map_err(|e| format!("modify_user_profile: {}", e.to_string()))?;

            if let Some(timer) = timer {
                info!("modify_user_profile() time: {}", format!("{:.2?}", timer.elapsed()));
            }
            Ok(result)
        }

        /// Filter for the list of stream logos by user ID.
        async fn filter_stream_logos(&self, user_id: i32) -> Result<Vec<String>, String> {
            let timer = if log_enabled!(Info) { Some(tm::now()) } else { None };

            let list: Vec<StreamLogo> = sqlx::query_as(
                "SELECT s.logo \
                FROM streams s \
                WHERE LENGTH(COALESCE(s.logo, '')) > 0 \
                  AND s.user_id = $1 ORDER BY s.id ASC \
                LIMIT 1000",
            )
            .bind(user_id)
            .fetch_all(&self.db_pool)
            .await
            .map_err(|e| format!("select_streams: {}", e.to_string()))?;

            let result = list.into_iter().map(|v| v.logo.clone()).collect();

            if let Some(timer) = timer {
                info!("filter_stream_logos() time: {}", format!("{:.2?}", timer.elapsed()));
            }
            Ok(result)
        }
    }
}

#[cfg(all(test, feature = "mockdata"))]
pub mod tests {

    use actix_web::web;
    use chrono::Utc;
    use vrb_authent::{
        config_jwt,
        user_db::tests::USER1_ID,
        user_models::{Profile, Session, User},
    };
    use vrb_common::consts;

    use crate::config_prfl;
    use crate::profile_db::ProfileDb;
    use crate::profile_models::{ModifyUserProfile, UserProfile};

    #[derive(Debug, Clone)]
    pub struct ProfileDbApp {
        pub user_profile_vec: Vec<UserProfile>,
        pub session_vec: Vec<Session>,
    }

    impl ProfileDbApp {
        /// Create a new instance.
        pub fn new() -> Self {
            ProfileDbApp {
                user_profile_vec: Vec::new(),
                session_vec: Vec::new(),
            }
        }
        /// Create a new instance with the specified profile list.
        /// Sessions are taken from "sessions", if it is empty, they are created automatically.
        #[cfg(test)]
        pub fn create(user_profiles: &[UserProfile]) -> Self {
            ProfileDbApp {
                user_profile_vec: user_profiles.to_vec(),
                session_vec: Vec::new(),
            }
        }
        #[rustfmt::skip]
        pub fn stream_logo_alias(user_id: i32) -> Option<String> {
            let idx = user_id - USER1_ID;
            if -1 < idx && idx < 4 { Some(format!("{}/file_logo_{}.png", consts::ALIAS_LOGO_FILES_DIR, idx)) } else { None }
        }
    }

    impl ProfileDb for ProfileDbApp {
        /// Get an entity (profile) by ID.
        async fn get_profile_by_id(&self, user_id: i32) -> Result<Option<Profile>, String> {
            let opt_user_profile = self
                .user_profile_vec
                .iter()
                .find(|profile| profile.user_id == user_id)
                .map(|profile| profile.clone());

            let opt_profile = opt_user_profile.map(|user_profile| Profile {
                user_id: user_profile.user_id,
                avatar: user_profile.avatar,
                descript: user_profile.descript,
                theme: user_profile.theme,
                locale: user_profile.locale,
                created_at: user_profile.created_at,
                updated_at: user_profile.updated_at,
            });
            Ok(opt_profile)
        }

        /// Get an entities (user, profile) by ID.
        async fn get_user_profile_by_id(&self, user_id: i32) -> Result<Option<UserProfile>, String> {
            let opt_user_profile = self
                .user_profile_vec
                .iter()
                .find(|profile| profile.user_id == user_id)
                .map(|profile| profile.clone());

            Ok(opt_user_profile)
        }

        /// Modify an entity (profile, user).
        async fn modify_user_profile(&self, user_id: i32, modify_user_profile: ModifyUserProfile) -> Result<Option<UserProfile>, String> {
            let opt_profile = self.user_profile_vec.iter().find(|profile| (*profile).user_id == user_id);
            let opt_profile3: Option<UserProfile> = if let Some(profile) = opt_profile {
                let profile2 = UserProfile {
                    user_id: profile.user_id,
                    nickname: modify_user_profile.nickname.unwrap_or(profile.nickname.clone()),
                    email: modify_user_profile.email.unwrap_or(profile.email.clone()),
                    role: modify_user_profile.role.unwrap_or(profile.role.clone()),
                    avatar: modify_user_profile.avatar.unwrap_or(profile.avatar.clone()),
                    descript: modify_user_profile.descript.or(profile.descript.clone()),
                    theme: modify_user_profile.theme.or(profile.theme.clone()),
                    locale: modify_user_profile.locale.or(profile.locale.clone()),
                    created_at: profile.created_at,
                    updated_at: Utc::now(),
                };
                Some(profile2)
            } else {
                None
            };
            Ok(opt_profile3)
        }

        /// Filter for the list of stream logos by user ID.
        async fn filter_stream_logos(&self, user_id: i32) -> Result<Vec<String>, String> {
            let mut result: Vec<String> = vec![];
            let opt_stream_logo = Self::stream_logo_alias(user_id);
            if opt_stream_logo.is_some() {
                result.push(opt_stream_logo.unwrap().clone());
            }
            Ok(result)
        }
    }

    pub struct ProfileDbTest {}

    impl ProfileDbTest {
        pub fn profiles(users: &[User]) -> Vec<UserProfile> {
            let profile_vec: Vec<UserProfile> = users.iter().map(|u| UserProfile::from(u.clone())).collect();
            profile_vec
        }
        pub fn cfg_config_jwt(config_jwt: config_jwt::ConfigJwt) -> impl FnOnce(&mut web::ServiceConfig) {
            move |config: &mut web::ServiceConfig| {
                let data_config_jwt = web::Data::new(config_jwt);
                config.app_data(web::Data::clone(&data_config_jwt));
            }
        }
        pub fn cfg_config_prfl(config_prfl: config_prfl::ConfigPrfl) -> impl FnOnce(&mut web::ServiceConfig) {
            move |config: &mut web::ServiceConfig| {
                let data_config_prfl = web::Data::new(config_prfl);
                config.app_data(web::Data::clone(&data_config_prfl));
            }
        }
        pub fn cfg_profile_db(data_p: Vec<UserProfile>) -> impl FnOnce(&mut web::ServiceConfig) {
            move |config: &mut web::ServiceConfig| {
                let data_user_profile_db = web::Data::new(ProfileDbApp::create(&data_p));

                config.app_data(web::Data::clone(&data_user_profile_db));
            }
        }
    }
}
