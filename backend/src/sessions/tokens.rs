use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::utils::parse_err;

pub const CD_INVALID_TOKEN: &str = "InvalidToken";
pub const CD_NUM_TOKEN_MIN: usize = 0;
pub const CD_NUM_TOKEN_MAX: usize = 10000;

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenClaims {
    pub sub: String,
    pub iat: usize,
    pub exp: usize,
}

pub fn create_token(
    sub: &str,
    secret: &[u8],
    expires_in_seconds: i64,
) -> Result<String, jsonwebtoken::errors::Error> {
    if sub.is_empty() {
        return Err(jsonwebtoken::errors::ErrorKind::InvalidSubject.into());
    }

    let now = Utc::now();
    let iat = now.timestamp() as usize;
    let exp = (now + Duration::minutes(expires_in_seconds)).timestamp() as usize;
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
        Err(_) => Err(CD_INVALID_TOKEN),
    }
}

pub fn generate_num_token() -> i32 {
    let mut rng = rand::thread_rng();
    let result = rng.gen_range(CD_NUM_TOKEN_MIN..CD_NUM_TOKEN_MAX);
    result as i32
}

pub fn create_dual_sub(user_id: i32, num_token: i32) -> String {
    format!("{}.{}", user_id, num_token)
}

pub fn parse_dual_sub(dual_sub: &str) -> Result<(i32, i32), String> {
    let list: Vec<&str> = dual_sub.split('.').collect();
    let user_id: &str = list.get(0).unwrap_or(&"").clone();
    let num_token: &str = list.get(1).unwrap_or(&"").clone();

    let user_id = user_id.parse::<i32>().map_err(|e| {
        let msg = parse_err::MSG_PARSE_INT_ERROR;
        format!("id: {} `{}` - {}", msg, user_id, e.to_string())
    })?;

    let num_token = num_token.parse::<i32>().map_err(|e| {
        let msg = parse_err::MSG_PARSE_INT_ERROR;
        format!("num_token: {} `{}` - {}", msg, num_token, e.to_string())
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

        let token = create_token(user_id, secret, 60).unwrap();
        let decoded_user_id = decode_token(&token, secret).unwrap().sub;

        assert_eq!(decoded_user_id, user_id);
    }

    #[test]
    fn test_create_token_with_empty_user_id() {
        let user_id = "";
        let secret = b"my-secret-key";

        let result = create_token(user_id, secret, 60);

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
        assert_eq!(result.unwrap_err(), CD_INVALID_TOKEN);
    }

    #[test]
    fn test_decode_expired_token() {
        let secret = b"my-secret-key";
        let expired_token = create_token("user123", secret, -60).unwrap();

        let result = decode_token(expired_token, secret);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), CD_INVALID_TOKEN);
    }
}
