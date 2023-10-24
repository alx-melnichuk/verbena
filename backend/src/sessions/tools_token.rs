use crate::errors::AppError;
use crate::utils::err;

use super::tokens;

pub fn collect_token(
    user_id: i32,
    num_token: i32,
    jwt_secret: &[u8],
    token_duration: i64,
) -> Result<String, String> {
    let sub = tokens::create_dual_sub(user_id, num_token);
    let token =
        tokens::create_token(&sub, &jwt_secret, token_duration).map_err(|e| e.to_string())?;

    Ok(token)
}

pub fn parse_token(token: &str, jwt_secret: &[u8]) -> Result<(i32, i32), AppError> {
    // Decode token and handle errors
    let token_claims = tokens::decode_token(token, jwt_secret).map_err(|e| {
        eprintln!("$^ parse_token() decode.is_err(): {}", e); // #-
        eprintln!("$^ parse_token() token: `{}`", token); // #-
        #[rustfmt::skip]
        log::debug!("{}: {} {}", err::CD_INVALID_TOKEN, err::MSG_INVALID_TOKEN, e);
        return AppError::new(err::CD_INVALID_TOKEN, err::MSG_INVALID_TOKEN).set_status(403);
    })?;

    let (user_id, num_token) = tokens::parse_dual_sub(&token_claims.sub).map_err(|e| {
        eprintln!("$^parse_dual_sub: {}", e.to_string()); // #-
        #[rustfmt::skip]
        log::debug!("{}: {}", err::CD_UNALLOWABLE_TOKEN, err::MSG_UNALLOWABLE_TOKEN);
        return AppError::new(err::CD_UNALLOWABLE_TOKEN, err::MSG_UNALLOWABLE_TOKEN)
            .set_status(403);
    })?;

    Ok((user_id, num_token))
}
