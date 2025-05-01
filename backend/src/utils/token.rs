use actix_web::{http, HttpRequest};

pub const BEARER: &str = "Bearer ";
pub const TOKEN_NAME: &str = "token";

/** Get the token value from the cookie of the http-request. */
pub fn get_token_from_cookie(request: &HttpRequest) -> Option<String> {
    // Attempt to extract token from cookie.
    request.cookie(TOKEN_NAME).map(|c| c.value().to_string())
}
/** Extract the JWT token from the request header. */
pub fn get_jwt_from_header(header_token: &str) -> Result<String, String> {
    const NO_AUTH_HEADER: &str = "No authentication header";

    if header_token.len() == 0 {
        return Err(NO_AUTH_HEADER.to_string());
    }
    let auth_header = match std::str::from_utf8(header_token.as_bytes()) {
        Ok(v) => v,
        Err(e) => return Err(format!("{} : {}", NO_AUTH_HEADER, e.to_string())),
    };
    if !auth_header.starts_with(BEARER) {
        return Err("Invalid authentication header".to_string());
    }
    Ok(auth_header.trim_start_matches(BEARER).to_owned())
}
/** Get the token value from the header of the http-request. */
pub fn get_token_from_header(request: &HttpRequest) -> Result<Option<String>, String> {
    // Attempt to extract token from authorization header.
    let opt_jwt_token = request
        .headers()
        .get(http::header::AUTHORIZATION)
        .map(|h| h.to_str().unwrap().to_string());

    if let Some(jwt_token) = opt_jwt_token {
        let res_token = get_jwt_from_header(&jwt_token)?;
        Ok(Some(res_token))
    } else {
        Ok(None::<String>)
    }
}
/** Get the token value from the cookie (header) of the http-request. */
pub fn get_token_from_cookie_or_header(request: &HttpRequest) -> Option<String> {
    let opt_token = get_token_from_cookie(request);
    opt_token.or(get_token_from_header(request).unwrap_or(None))
}
