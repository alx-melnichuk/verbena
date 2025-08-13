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

pub fn get_test_config() -> ConfigJwt {
    ConfigJwt {
        jwt_secret: "my-jwt-secret".to_string(),
        jwt_access: 120,
        jwt_refresh: 240,
    }
}
