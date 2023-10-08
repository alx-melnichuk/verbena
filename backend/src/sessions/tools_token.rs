use crate::{errors::AppError, sessions::tokens, utils::err};

pub fn parse_token(token: &str, jwt_secret: &[u8]) -> Result<(i32, i32), AppError> {
    // Decode token and handle errors
    let token_claims = tokens::decode_token(token, jwt_secret).map_err(|e| {
        eprintln!("$^decode.is_err()"); // #-
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
