use std::env;

pub const ACCESS_TOKEN_DURATION: &str = "900"; // 900 seconds = 15 minutes
pub const REFRESH_TOKEN_DURATION: &str = "345600"; // 345600 seconds = 4 days  60sec*60min*24hour*4days

#[derive(Debug, Clone)]
pub struct ConfigJwt {
    pub jwt_secret: String,
    // access token duration in seconds
    pub jwt_access: i64,
    // refresh token duration in seconds
    pub jwt_refresh: i64,
}

impl ConfigJwt {
    pub fn init_by_env() -> Self {
        let jwt_secret = env::var("JWT_SECRET_KEY").expect("Env \"JWT_SECRET_KEY\" not found.");
        let jwt_access = env::var("JWT_ACCESS_TOKEN_DURATION").unwrap_or(ACCESS_TOKEN_DURATION.to_string());
        let jwt_refresh = env::var("JWT_REFRESH_TOKEN_DURATION").unwrap_or(REFRESH_TOKEN_DURATION.to_string());

        ConfigJwt {
            jwt_secret,
            jwt_access: jwt_access.parse::<i64>().unwrap(),
            jwt_refresh: jwt_refresh.parse::<i64>().unwrap(),
        }
    }
}

#[cfg(any(test, feature = "mockdata"))]
pub mod tests {
    use actix_web::web;
    use vrb_tools::token_coding;

    use crate::config_jwt::ConfigJwt;

    pub fn get_config() -> ConfigJwt {
        ConfigJwt {
            jwt_secret: "my-jwt-secret".to_string(),
            jwt_access: 120,
            jwt_refresh: 240,
        }
    }
    pub fn get_num_token(user_id: i32) -> i32 {
        40000 + user_id
    }
    pub fn get_token(user_id: i32) -> String {
        let config_jwt = get_config();
        let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();
        let num_token = get_num_token(user_id);
        token_coding::encode_token(user_id, num_token, &jwt_secret, config_jwt.jwt_access).unwrap()
    }
    pub fn cfg_config_jwt(config_jwt: ConfigJwt) -> impl FnOnce(&mut web::ServiceConfig) {
        move |config: &mut web::ServiceConfig| {
            let data_config_jwt = web::Data::new(config_jwt);
            config.app_data(web::Data::clone(&data_config_jwt));
        }
    }
}
