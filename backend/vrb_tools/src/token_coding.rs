use std::fmt::Debug;

use chrono::{Duration, Utc};
use jsonwebtoken::{self as jwt, errors};
use log::error;
use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::{crypto, parser};

pub const CD_NUM_TOKEN_MIN: usize = 1;
pub const CD_NUM_TOKEN_MAX: usize = 10000;
// User_ID from the header does not match the user_ID from the parameters
pub const CD_UNALLOWABLE_TOKEN: &str = "UnallowableToken";

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
) -> Result<String, String> {
    if num_token == 0 {
        let err = errors::Error::from(errors::ErrorKind::InvalidSubject).to_string();
        error!("{:?}", err);
        return Err(err);
    }
    if secret.len() == 0 {
        let err = errors::Error::from(errors::ErrorKind::InvalidKeyFormat).to_string();
        error!("{:?}", err);
        return Err(err);
    }

    let now = Utc::now();
    let iat = now.timestamp() as usize;
    let exp = (now + Duration::seconds(expires)).timestamp() as usize;
    let iss = num_token.to_string();
    let sub = user_id.to_string();

    let claims = TokenClaims { exp, iat, iss, sub };
    // Encode the header and claims given and sign the payload using the algorithm from the header and the key.
    #[rustfmt::skip]
    let encoded = jwt::encode(
        &jwt::Header::new(jwt::Algorithm::HS256),
        &claims, &jwt::EncodingKey::from_secret(secret)
    ).map_err(|e| {
        let err = e.to_string();
        error!("{:?}", err);
        err
    })?;
    // Encrypt the data using a secret string.
    let encrypted = crypto::encrypt_aes(secret, encoded.as_bytes())?;

    Ok(encrypted)
}

/// Unpack two parameters from the token.
pub fn decode_token<T: Into<String>>(token: T, secret: &[u8]) -> Result<(i32, i32), String> {
    if secret.len() == 0 {
        let err = errors::Error::from(errors::ErrorKind::InvalidKeyFormat).to_string();
        error!("{:?}", err);
        return Err(err);
    }

    let token_into: String = token.into();
    if token_into.len() == 0 {
        let err = errors::Error::from(errors::ErrorKind::InvalidSubject).to_string();
        error!("{:?}", err);
        return Err(err);
    }

    let decrypted = crypto::decrypt_aes(secret, &token_into).map_err(|err| {
        error!("{:?}", err.to_string());
        err.to_string()
    })?;

    let token_str = std::str::from_utf8(&decrypted).map_err(|err| {
        error!("{:?}", err.to_string());
        err.to_string()
    })?;
    #[rustfmt::skip]
    let token_data = jwt::decode::<TokenClaims>(
        token_str,
        &jwt::DecodingKey::from_secret(secret),
        &jwt::Validation::new(jwt::Algorithm::HS256)
    )
    .map_err(|err| {
        error!("{:?}", err.to_string());
        err.to_string()
    })?;

    let user_id_str = token_data.claims.sub.as_str();
    let num_token_str = token_data.claims.iss.as_str();

    let user_id = parser::parse_i32(user_id_str).map_err(|err| {
        #[rustfmt::skip]
        error!("{}: user_id: {} - {}", CD_UNALLOWABLE_TOKEN, user_id_str, err);
        CD_UNALLOWABLE_TOKEN
    })?;

    let num_token = parser::parse_i32(num_token_str).map_err(|err| {
        #[rustfmt::skip]
        error!("{}: num_token: {} - {}", CD_UNALLOWABLE_TOKEN, num_token_str, err);
        CD_UNALLOWABLE_TOKEN
    })?;

    Ok((user_id, num_token))
}

pub fn generate_num_token() -> i32 {
    let mut rng = rand::rng();
    let result = rng.random_range(CD_NUM_TOKEN_MIN..CD_NUM_TOKEN_MAX);
    result as i32
}

#[cfg(test)]
mod tests {

    use super::*;

    const EXPIRES: i64 = 61;

    // ** encode_token **

    #[test]
    fn test_encode_with_0_to_num_token() {
        let user_id: i32 = 123;
        let num_token: i32 = 0;
        let secret = b"super-secret-key";

        let result = encode_token(user_id, num_token, secret, EXPIRES);

        assert!(result.is_err());
        let r = jwt::errors::Error::from(jwt::errors::ErrorKind::InvalidSubject).to_string();
        assert_eq!(result.unwrap_err(), r);
    }
    #[test]
    fn test_encode_with_empty_secret() {
        let user_id: i32 = 123;
        let num_token: i32 = 567;
        let secret = b"";

        let result = encode_token(user_id, num_token, secret, EXPIRES);

        assert!(result.is_err());
        let r = jwt::errors::Error::from(jwt::errors::ErrorKind::InvalidKeyFormat).to_string();
        assert_eq!(result.unwrap_err(), r);
    }

    // ** decode_token **

    #[test]
    fn test_decode_with_empty_secret() {
        let token = "value-token";
        let secret = b"";

        let result = decode_token(token, secret);

        assert!(result.is_err());
        let r = jwt::errors::Error::from(jwt::errors::ErrorKind::InvalidKeyFormat).to_string();
        assert_eq!(result.unwrap_err(), r);
    }
    #[test]
    fn test_decode_with_empty_token() {
        let token = "";
        let secret = b"super-secret-key";

        let result = decode_token(token, secret);

        assert!(result.is_err());
        let r = jwt::errors::Error::from(jwt::errors::ErrorKind::InvalidSubject).to_string();
        assert_eq!(result.unwrap_err(), r);
    }
    #[test]
    fn test_decode_with_bad_token() {
        let token = "bad";
        let secret = b"super-secret-key";

        let result = decode_token(token, secret);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), crypto::CRT_WRONG_STRING_BASE64URL);
    }
    #[test]
    fn test_decode_expired_token() {
        let user_id: i32 = 123;
        let num_token: i32 = 567;
        let secret = b"super-secret-key";
        let expired_token = encode_token(user_id, num_token, secret, -EXPIRES).unwrap();

        let result = decode_token(expired_token, secret);

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "ExpiredSignature");
    }
    #[test]
    fn test_encode_and_decoded_valid_token() {
        let user_id: i32 = 123;
        let num_token: i32 = 567;
        let secret = b"super-secret-key";

        let token = encode_token(user_id, num_token, secret, EXPIRES).unwrap();
        let (res_user_id, res_num_token) = decode_token(&token, secret).unwrap();

        assert_eq!(res_user_id, user_id);
        assert_eq!(res_num_token, num_token);
    }
}
