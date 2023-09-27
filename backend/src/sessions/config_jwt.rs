#[derive(Debug, Clone)]
pub struct ConfigJwt {
    pub jwt_secret: String,
    pub jwt_maxage: i64, // # maximum duration
}

impl ConfigJwt {
    pub fn init_by_env() -> Self {
        let jwt_secret = std::env::var("JWT_SECRET_KEY").expect("JWT_SECRET_KEY must be set");
        let jwt_maxage = std::env::var("JWT_MAXAGE").expect("JWT_MAXAGE must be set");

        ConfigJwt {
            jwt_secret,
            jwt_maxage: jwt_maxage.parse::<i64>().unwrap(),
        }
    }
}

// #[allow(dead_code)]
#[cfg(feature = "mockdata")]
pub fn get_test_config() -> ConfigJwt {
    ConfigJwt {
        jwt_secret: "my-jwt-secret".to_string(),
        jwt_maxage: 60,
    }
}
