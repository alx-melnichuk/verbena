use chrono::{Duration, Utc};
use jsonwebtoken as jwt;
use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::settings::err;
use crate::utils::parser;

pub const CD_NUM_TOKEN_MIN: usize = 1;
pub const CD_NUM_TOKEN_MAX: usize = 10000;

#[derive(Debug, Serialize, Deserialize)]
struct TokenClaims {
    pub exp: usize,
    pub iat: usize,
    pub iss: String,
    pub sub: String,
}

/// Pack two parameters into a token.
pub fn encode_token(
    user_id: i32,
    num_token: i32,
    secret: &[u8],
    // expires in seconds
    expires: i64,
) -> Result<String, jwt::errors::Error> {
    if num_token == 0 {
        return Err(jwt::errors::ErrorKind::InvalidSubject.into());
    }
    if secret.len() == 0 {
        return Err(jwt::errors::ErrorKind::InvalidEcdsaKey.into());
    }

    let now = Utc::now();
    let iat = now.timestamp() as usize;
    let exp = (now + Duration::seconds(expires)).timestamp() as usize;
    let iss = num_token.to_string();
    let sub = user_id.to_string();

    let claims = TokenClaims { exp, iat, iss, sub };

    jwt::encode(
        &jwt::Header::new(jwt::Algorithm::HS256),
        &claims,
        &jwt::EncodingKey::from_secret(secret),
    )
}

/// Unpack two parameters from the token.
pub fn decode_token<T: Into<String>>(token: T, secret: &[u8]) -> Result<(i32, i32), String> {
    let token_data = jwt::decode::<TokenClaims>(
        &token.into(),
        &jwt::DecodingKey::from_secret(secret),
        &jwt::Validation::new(jwt::Algorithm::HS256),
    )
    .map_err(|err| {
        log::error!("decode error: {:?}", err);
        err::CD_FORBIDDEN
    })?;

    let user_id_str = token_data.claims.sub.as_str();
    let num_token_str = token_data.claims.iss.as_str();

    let user_id = parser::parse_i32(user_id_str).map_err(|err| {
        #[rustfmt::skip]
        log::error!("{}: user_id: {} - {}", err::CD_UNALLOWABLE_TOKEN, user_id_str, err);
        err::CD_UNALLOWABLE_TOKEN
    })?;

    let num_token = parser::parse_i32(num_token_str).map_err(|err| {
        #[rustfmt::skip]
        log::error!("{}: num_token: {} - {}", err::CD_UNALLOWABLE_TOKEN, num_token_str, err);
        err::CD_UNALLOWABLE_TOKEN
    })?;

    Ok((user_id, num_token))
}

pub fn generate_num_token() -> i32 {
    let mut rng = rand::thread_rng();
    let result = rng.gen_range(CD_NUM_TOKEN_MIN..CD_NUM_TOKEN_MAX);
    result as i32
}

#[cfg(test)]
mod tests {

    use super::*;

    const EXPIRES: i64 = 61;

    #[test]
    fn test2_encode_with_0_to_num_token() {
        let user_id: i32 = 123;
        let num_token: i32 = 0;
        let secret = b"super-secret-key";

        let result = encode_token(user_id, num_token, secret, EXPIRES);

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().into_kind(),
            jwt::errors::ErrorKind::InvalidSubject
        );
    }
    #[test]
    fn test2_encode_with_empty_secret() {
        let user_id: i32 = 123;
        let num_token: i32 = 567;
        let secret = b"";

        let result = encode_token(user_id, num_token, secret, EXPIRES);

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().into_kind(),
            jwt::errors::ErrorKind::InvalidEcdsaKey
        );
    }
    #[test]
    fn test2_encode_and_decoded_valid_token() {
        let user_id: i32 = 123;
        let num_token: i32 = 567;
        let secret = b"super-secret-key";

        let token = encode_token(user_id, num_token, secret, EXPIRES).unwrap();
        let (res_user_id, res_num_token) = decode_token(&token, secret).unwrap();

        assert_eq!(res_user_id, user_id);
        assert_eq!(res_num_token, num_token);
    }

    #[test]
    fn test2_decoded_invalid_token() {
        let secret = b"super-secret-key";
        let invalid_token = "invalid-token";

        let result = decode_token(invalid_token, secret);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), err::CD_FORBIDDEN);
    }
    #[test]
    fn test2_decode_expired_token() {
        let user_id: i32 = 123;
        let num_token: i32 = 567;
        let secret = b"super-secret-key";

        let expired_token = encode_token(user_id, num_token, secret, -EXPIRES).unwrap();
        let result = decode_token(expired_token, secret);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), err::CD_FORBIDDEN);
    }
}
