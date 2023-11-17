use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::errors::AppError;
use crate::settings::err::{
    CD_FORBIDDEN, CD_UNALLOWABLE_TOKEN, MSG_INVALID_OR_EXPIRED_TOKEN, MSG_UNALLOWABLE_TOKEN,
};
use crate::utils::parser;

pub const CD_NUM_TOKEN_MIN: usize = 1;
pub const CD_NUM_TOKEN_MAX: usize = 10000;

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenClaims {
    pub sub: String,
    pub iat: usize,
    pub exp: usize,
}

pub fn encode_token(
    sub: &str,
    secret: &[u8],
    // expires in minutes
    expires: i64,
) -> Result<String, jsonwebtoken::errors::Error> {
    if sub.is_empty() {
        return Err(jsonwebtoken::errors::ErrorKind::InvalidSubject.into());
    }

    let now = Utc::now();
    let iat = now.timestamp() as usize;
    let exp = (now + Duration::minutes(expires)).timestamp() as usize;
    let sub = sub.to_string();

    let claims: TokenClaims = TokenClaims { sub, exp, iat };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret),
    )
}

pub fn decode_token<T: Into<String>>(token: T, secret: &[u8]) -> Result<TokenClaims, &str> {
    let decoded = decode::<TokenClaims>(
        &token.into(),
        &DecodingKey::from_secret(secret),
        &Validation::new(Algorithm::HS256),
    );

    match decoded {
        Ok(token) => Ok(token.claims),
        Err(_) => Err(CD_FORBIDDEN),
    }
}

pub fn generate_num_token() -> i32 {
    let mut rng = rand::thread_rng();
    let result = rng.gen_range(CD_NUM_TOKEN_MIN..CD_NUM_TOKEN_MAX);
    result as i32
}

/// Pack two parameters into a token.
pub fn encode_dual_token(
    user_id: i32,
    num_token: i32,
    jwt_secret: &[u8],
    // token duration in minutes
    duration: i64,
) -> Result<String, String> {
    let sub = format!("{}.{}", user_id, num_token);
    let token = encode_token(&sub, &jwt_secret, duration).map_err(|e| e.to_string())?;

    Ok(token)
}

/// Unpack two parameters from the token.
pub fn decode_dual_token(token: &str, jwt_secret: &[u8]) -> Result<(i32, i32), AppError> {
    let token_claims = decode_token(token, jwt_secret).map_err(|e| {
        #[rustfmt::skip]
        log::error!("{}: {} {}", CD_FORBIDDEN, MSG_INVALID_OR_EXPIRED_TOKEN, e);
        return AppError::new(CD_FORBIDDEN, MSG_INVALID_OR_EXPIRED_TOKEN).set_status(403);
    })?;

    let list: Vec<&str> = token_claims.sub.split('.').collect();
    let user_id_str: &str = list.get(0).unwrap_or(&"");
    let num_token_str: &str = list.get(1).unwrap_or(&"");

    let user_id = parser::parse_i32(user_id_str).map_err(|err| {
        log::error!("{CD_UNALLOWABLE_TOKEN}: {MSG_UNALLOWABLE_TOKEN} - id: {err}");
        return AppError::new(CD_UNALLOWABLE_TOKEN, MSG_UNALLOWABLE_TOKEN).set_status(403);
    })?;

    let num_token = parser::parse_i32(num_token_str).map_err(|err| {
        log::error!("{CD_UNALLOWABLE_TOKEN}: {MSG_UNALLOWABLE_TOKEN} - num_token: {err}");
        return AppError::new(CD_UNALLOWABLE_TOKEN, MSG_UNALLOWABLE_TOKEN).set_status(403);
    })?;

    Ok((user_id, num_token))
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_create_and_decoded_valid_token() {
        let user_id = "user123";
        let secret = b"my-secret-key";

        let token = encode_token(user_id, secret, 60).unwrap();
        let decoded_user_id = decode_token(&token, secret).unwrap().sub;

        assert_eq!(decoded_user_id, user_id);
    }

    #[test]
    fn test_create_token_with_empty_user_id() {
        let user_id = "";
        let secret = b"my-secret-key";

        let result = encode_token(user_id, secret, 60);

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().into_kind(),
            jsonwebtoken::errors::ErrorKind::InvalidSubject
        );
    }

    #[test]
    fn test_decoded_invalid_token() {
        let secret = b"my-secret-key";
        let invalid_token = "invalid-token";

        let result = decode_token(invalid_token, secret);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), CD_FORBIDDEN);
    }

    #[test]
    fn test_decode_expired_token() {
        let secret = b"my-secret-key";
        let expired_token = encode_token("user123", secret, -60).unwrap();

        let result = decode_token(expired_token, secret);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), CD_FORBIDDEN);
    }
}
