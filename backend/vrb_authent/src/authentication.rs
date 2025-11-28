use std::{rc::Rc, time::Instant as tm};

use actix_web::{FromRequest, HttpMessage, dev, error, http::StatusCode, web};
use futures_util::{
    FutureExt,
    future::{LocalBoxFuture, Ready, ready},
};
use log::{Level::Info, error, info, log_enabled};
use vrb_common::{
    api_error::{ApiError, code_to_str},
    err,
};
use vrb_dbase::enm_user_role::UserRole;
use vrb_tools::{token_coding, token_data};

#[cfg(not(any(test, feature = "mockdata")))]
use crate::user_orm::impls::UserOrmApp;
#[cfg(any(test, feature = "mockdata"))]
use crate::user_orm::tests::UserOrmApp;
use crate::{
    config_jwt,
    user_models::{Session, User},
    user_orm::UserOrm,
};

// 500 Internal Server Error - Authentication: The entity "user" was not received from the request.
pub const MSG_USER_NOT_RECEIVED_FROM_REQUEST: &str = "user_not_received_from_request";

pub struct Authenticated(User);

impl FromRequest for Authenticated {
    type Error = actix_web::Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &actix_web::HttpRequest, _payload: &mut dev::Payload) -> Self::Future {
        let value = req.extensions().get::<User>().cloned();
        let result = match value {
            Some(user) => Ok(Authenticated(user)),
            None => Err(error::ErrorInternalServerError(ApiError::new(500, MSG_USER_NOT_RECEIVED_FROM_REQUEST))),
        };
        ready(result)
    }
}

impl std::ops::Deref for Authenticated {
    type Target = User;
    /// Implement the deref method to access the inner "User" value of Authenticated.
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct RequireAuth {
    pub allowed_roles: Rc<Vec<UserRole>>,
}

impl RequireAuth {
    #[allow(dead_code)]
    pub fn allowed_roles(allowed_roles: Vec<UserRole>) -> Self {
        RequireAuth {
            allowed_roles: Rc::new(allowed_roles),
        }
    }
    pub fn all_roles() -> Vec<UserRole> {
        vec![UserRole::User, UserRole::Moderator, UserRole::Admin]
    }
    pub fn admin_role() -> Vec<UserRole> {
        vec![UserRole::Admin]
    }
}

impl<S> dev::Transform<S, dev::ServiceRequest> for RequireAuth
where
    S: dev::Service<dev::ServiceRequest, Response = dev::ServiceResponse<actix_web::body::BoxBody>, Error = actix_web::Error> + 'static,
{
    /// The response type produced by the service.
    type Response = dev::ServiceResponse<actix_web::body::BoxBody>;
    /// The error type produced by the service.
    type Error = actix_web::Error;
    /// The `TransformService` value created by this factory.
    type Transform = AuthMiddleware<S>;
    /// Errors produced while building a transform service.
    type InitError = ();
    /// The future type representing the asynchronous response.
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    /// Creates and returns a new Transform component asynchronously.
    /// A `Self::Future` representing the asynchronous transformation process.
    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthMiddleware {
            service: Rc::new(service),
            allowed_roles: self.allowed_roles.clone(),
        }))
    }
}

pub struct AuthMiddleware<S> {
    service: Rc<S>,
    allowed_roles: Rc<Vec<UserRole>>,
}

impl<S> dev::Service<dev::ServiceRequest> for AuthMiddleware<S>
where
    S: dev::Service<dev::ServiceRequest, Response = dev::ServiceResponse<actix_web::body::BoxBody>, Error = actix_web::Error> + 'static,
{
    /// The response type produced by the service.
    type Response = dev::ServiceResponse<actix_web::body::BoxBody>;
    /// The error type that can be produced by the service.
    type Error = actix_web::Error;
    /// The future type representing the asynchronous response.
    type Future = LocalBoxFuture<'static, Result<Self::Response, actix_web::Error>>;

    // Returns `Ready` when the service is able to process requests.
    dev::forward_ready!(service);
    // fn poll_ready(&self, ctx: &mut core::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
    //     self.service.poll_ready(ctx)
    // }

    /// The future type representing the asynchronous response.
    fn call(&self, req: dev::ServiceRequest) -> Self::Future {
        let timer = if log_enabled!(Info) { Some(tm::now()) } else { None };

        // Extract the token from the cookie or authorization header.
        let token = token_data::get_token_from_cookie_or_header(req.request());

        // If the token is missing, then an error (code: "Unauthorized", message: "token_missing").
        if token.is_none() {
            error!("{}: {}", code_to_str(StatusCode::UNAUTHORIZED), err::MSG_MISSING_TOKEN);
            let json_error = ApiError::new(401, err::MSG_MISSING_TOKEN);
            return Box::pin(ready(Err(error::ErrorUnauthorized(json_error)))); // 401(a)
        }
        let token = token.unwrap().clone();

        let config_jwt = req.app_data::<web::Data<config_jwt::ConfigJwt>>().unwrap();
        let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();
        // Decode token (check token lifetime validity).
        let token_res = token_coding::decode_token(&token, jwt_secret);

        if let Err(e) = token_res {
            let message = format!("{}: {}", err::MSG_INVALID_OR_EXPIRED_TOKEN, &e);
            error!("{}: {}", code_to_str(StatusCode::UNAUTHORIZED), &message);
            let json_error = ApiError::new(401, &message);
            return Box::pin(ready(Err(error::ErrorUnauthorized(json_error)))); // 401(b)
        }

        let (user_id, num_token) = token_res.unwrap();

        let allowed_roles = self.allowed_roles.clone();
        let srv = Rc::clone(&self.service);

        // Handle user extraction and request processing
        async move {
            let user_orm = req.app_data::<web::Data<UserOrmApp>>().unwrap().get_ref();

            // Token verification:
            // 1. Search for a session by "id" from the token;
            let opt_session = user_orm.get_session_by_id(user_id).map_err(|e| {
                error!("{}-{}; {}", code_to_str(StatusCode::INSUFFICIENT_STORAGE), err::MSG_DATABASE, &e);
                return ApiError::create(507, err::MSG_DATABASE, &e); // 507
            })?;
            // If the session does not exist, return error 406("NotAcceptable", "session_not_found; user_id: {}").
            let session = is_session_not_found(opt_session, user_id)?;
            // 2. Compare "num_token" from session with "num_token" from token;
            // To block hacking, the session contains a numeric value "num_token".
            // If session.num_token is not equal to token.num_token,return error401(c)("Unauthorized","unacceptable_token_num; user_id: {}")
            let _ = is_unacceptable_token_num(&session, num_token, user_id)?;
            // 3. If everything is correct, then search for the user by "user_id" from the token;
            let opt_user = user_orm.get_user_by_id(user_id, false).map_err(|e| {
                error!("{}-{}; {}", code_to_str(StatusCode::INSUFFICIENT_STORAGE), err::MSG_DATABASE, &e);
                ApiError::create(507, err::MSG_DATABASE, &e) // 507
            })?;
            // If the user is not present, return error401(d)("Unauthorized", "unacceptable_token_id; user_id: {}").
            let user = is_unacceptable_token_id(opt_user, user_id)?;

            if let Some(timer) = timer {
                info!("authentication() time: {}", format!("{:.2?}", timer.elapsed()));
            }

            // Check if user's role matches the required role
            if allowed_roles.contains(&user.role) {
                // Insert user information into request extensions.
                req.extensions_mut().insert::<User>(user);
                // Call the wrapped service to handle the request
                let res = srv.call(req).await?;
                Ok(res)
            } else {
                error!("{}: {}", code_to_str(StatusCode::FORBIDDEN), err::MSG_ACCESS_DENIED);
                let err_msg = ApiError::new(403, err::MSG_ACCESS_DENIED);
                Err(error::ErrorForbidden(err_msg)) // 403
            }
        }
        .boxed_local()
    }
}

/// If the session is missing, then return an error406("NotAcceptable", "session_not_found; user_id: {}").
pub fn is_session_not_found(opt_session: Option<Session>, user_id: i32) -> Result<Session, ApiError> {
    let session = opt_session.ok_or_else(|| {
        // There is no session for this user.
        let msg = format!("user_id: {}", user_id);
        error!("{}-{}; {}", code_to_str(StatusCode::NOT_ACCEPTABLE), err::MSG_SESSION_NOT_FOUND, &msg);
        ApiError::create(406, err::MSG_SESSION_NOT_FOUND, &msg) // 406
    })?;
    Ok(session)
}
pub fn is_unacceptable_token_num(session: &Session, num_token: i32, user_id: i32) -> Result<(), ApiError> {
    // Each session contains an additional numeric value.
    // Compare an additional numeric value from the session and from the token.
    if session.num_token.is_none() || session.num_token.unwrap() != num_token {
        // If they do not match, then this is an error.
        let msg = format!("user_id: {}", user_id);
        error!("{}-{}; {}", code_to_str(StatusCode::UNAUTHORIZED), err::MSG_UNACCEPTABLE_TOKEN_NUM, &msg); // 401(c)
        return Err(ApiError::create(401, err::MSG_UNACCEPTABLE_TOKEN_NUM, &msg));
    }
    Ok(())
}
/// If the user is missing, then return an error401(d)("Unauthorized", "unacceptable_token_id; user_id: {}").
pub fn is_unacceptable_token_id(opt_user: Option<User>, user_id: i32) -> Result<User, ApiError> {
    let user = opt_user.ok_or_else(|| {
        let msg = format!("user_id: {}", user_id);
        error!("{}-{}; {}", code_to_str(StatusCode::UNAUTHORIZED), err::MSG_UNACCEPTABLE_TOKEN_ID, &msg);
        ApiError::create(401, err::MSG_UNACCEPTABLE_TOKEN_ID, &msg) // 401(d)
    })?;
    Ok(user)
}
