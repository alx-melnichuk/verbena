#[derive(Debug, Clone)]
pub struct ConfigJwt {
    pub jwt_secret: String,
    // maximum duration in seconds
    pub jwt_maxage: i64,
    // access token duration in seconds
    pub jwt_access: i64,
    // refresh token duration in seconds
    pub jwt_refresh: i64,
}

impl ConfigJwt {
    pub fn init_by_env() -> Self {
        let jwt_secret = std::env::var("JWT_SECRET_KEY").expect("JWT_SECRET_KEY must be set");
        let jwt_maxage = std::env::var("JWT_MAXAGE").expect("JWT_MAXAGE must be set");
        let jwt_access = std::env::var("JWT_ACCESS_TOKEN_DURATION")
            .expect("JWT_ACCESS_TOKEN_DURATION must be set");
        let jwt_refresh = std::env::var("JWT_REFRESH_TOKEN_DURATION")
            .expect("JWT_REFRESH_TOKEN_DURATION must be set");

        ConfigJwt {
            jwt_secret,
            jwt_maxage: jwt_maxage.parse::<i64>().unwrap(),
            jwt_access: jwt_access.parse::<i64>().unwrap(),
            jwt_refresh: jwt_refresh.parse::<i64>().unwrap(),
        }
    }
}

#[cfg(all(test, feature = "mockdata"))]
pub fn get_test_config() -> ConfigJwt {
    ConfigJwt {
        jwt_secret: "my-jwt-secret".to_string(),
        jwt_maxage: 360,
        jwt_access: 120,
        jwt_refresh: 240,
    }
}
