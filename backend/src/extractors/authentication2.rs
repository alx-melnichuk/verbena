use std::rc::Rc;
use std::time::Instant as tm;

use actix_web::{dev, error, http::StatusCode, web, FromRequest, HttpMessage};
use futures_util::{
    future::{ready, LocalBoxFuture, Ready},
    FutureExt,
};
use log::{debug, error, info, log_enabled, Level::Info};
use vrb_common::api_error::{code_to_str, ApiError};
#[cfg(not(all(test, feature = "mockdata")))]
use vrb_dbase::user_auth_orm::impls::UserAuthOrmApp;
#[cfg(all(test, feature = "mockdata"))]
use vrb_dbase::user_auth_orm::tests::UserAuthOrmApp;
use vrb_dbase::{config_jwt, db_enums::UserRole, user_auth_models::User, user_auth_orm::UserAuthOrm};
use vrb_tools::{err, token_coding, token_data};


// 500 Internal Server Error - Authentication: The entity "user" was not received from the request.
pub const MSG_USER_NOT_RECEIVED_FROM_REQUEST: &str = "user_not_received_from_request";

pub struct Authenticated2(User);

impl FromRequest for Authenticated2 {
    type Error = actix_web::Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &actix_web::HttpRequest, _payload: &mut dev::Payload) -> Self::Future {
        let value = req.extensions().get::<User>().cloned();
        let result = match value {
            Some(user) => Ok(Authenticated2(user)),
            None => Err(error::ErrorInternalServerError(ApiError::new(500, MSG_USER_NOT_RECEIVED_FROM_REQUEST))),
        };
        ready(result)
    }
}

impl std::ops::Deref for Authenticated2 {
    type Target = User;
    /// Implement the deref method to access the inner "User" value of Authenticated.
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct RequireAuth2 {
    pub allowed_roles: Rc<Vec<UserRole>>,
}

impl RequireAuth2 {
    #[allow(dead_code)]
    pub fn allowed_roles(allowed_roles: Vec<UserRole>) -> Self {
        RequireAuth2 {
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

impl<S> dev::Transform<S, dev::ServiceRequest> for RequireAuth2
where
    S: dev::Service<dev::ServiceRequest, Response = dev::ServiceResponse<actix_web::body::BoxBody>, Error = actix_web::Error> + 'static,
{
    /// The response type produced by the service.
    type Response = dev::ServiceResponse<actix_web::body::BoxBody>;
    /// The error type produced by the service.
    type Error = actix_web::Error;
    /// The `TransformService` value created by this factory.
    type Transform = AuthMiddleware2<S>;
    /// Errors produced while building a transform service.
    type InitError = ();
    /// The future type representing the asynchronous response.
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    /// Creates and returns a new Transform component asynchronously.
    /// A `Self::Future` representing the asynchronous transformation process.
    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthMiddleware2 {
            service: Rc::new(service),
            allowed_roles: self.allowed_roles.clone(),
        }))
    }
}

pub struct AuthMiddleware2<S> {
    service: Rc<S>,
    allowed_roles: Rc<Vec<UserRole>>,
}

impl<S> dev::Service<dev::ServiceRequest> for AuthMiddleware2<S>
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
        let opt_timer0 = if log_enabled!(Info) { Some(std::time::Instant::now()) } else { None };

        // Attempt to extract token from cookie or authorization header
        let token = token_data::get_token_from_cookie_or_header(req.request());

        // If token is missing, return unauthorized error
        if token.is_none() {
            error!("{}: {}", code_to_str(StatusCode::UNAUTHORIZED), err::MSG_MISSING_TOKEN);
            let json_error = ApiError::new(401, err::MSG_MISSING_TOKEN);
            return Box::pin(ready(Err(error::ErrorUnauthorized(json_error)))); // 401(a)
        }
        let token = token.unwrap().clone();

        let config_jwt = req.app_data::<web::Data<config_jwt::ConfigJwt>>().unwrap();
        let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();
        // Decode the token.
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
            let user_auth_orm = req.app_data::<web::Data<UserAuthOrmApp>>().unwrap().get_ref();
            // Check the token for correctness and get the user.
            let res_user = check_token_and_get_user(user_id, num_token, user_auth_orm).await;
            if let Err(app_error) = res_user {
                return Err(app_error.into());
            }
            let user = res_user.unwrap();

            if let Some(timer0) = opt_timer0 {
                debug!("timer0: {}, user.role: {:?}", format!("{:.2?}", timer0.elapsed()), &user.role);
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

/** Check the token for correctness and get the user. */
pub async fn check_token_and_get_user(user_id: i32, num_token: i32, user_auth_orm: &UserAuthOrmApp) -> Result<User, ApiError> {
    let timer = if log_enabled!(Info) { Some(tm::now()) } else { None };

    // Find a session for a given user.
    let opt_session = user_auth_orm.get_session_by_id(user_id).map_err(|e| {
        error!("{}-{}; {}", code_to_str(StatusCode::INSUFFICIENT_STORAGE), err::MSG_DATABASE, &e);
        return ApiError::create(507, err::MSG_DATABASE, &e); // 507
    })?;
    let session = opt_session.ok_or_else(|| {
        // There is no session for this user.
        let msg = format!("user_id: {}", user_id);
        error!("{}-{}; {}", code_to_str(StatusCode::NOT_ACCEPTABLE), err::MSG_SESSION_NOT_FOUND, &msg);
        ApiError::create(406, err::MSG_SESSION_NOT_FOUND, &msg) // 406
    })?;
    // Each session contains an additional numeric value.
    let session_num_token = session.num_token.unwrap_or(0);
    // Compare an additional numeric value from the session and from the token.
    if session_num_token != num_token {
        // If they do not match, then this is an error.
        let msg = format!("user_id: {}", user_id);
        error!("{}-{}; {}", code_to_str(StatusCode::UNAUTHORIZED), err::MSG_UNACCEPTABLE_TOKEN_NUM, &msg); // 401(c)
        return Err(ApiError::create(401, err::MSG_UNACCEPTABLE_TOKEN_NUM, &msg));
    }
    let result = user_auth_orm.get_user_by_id(user_id, false).map_err(|e| {
        error!("{}-{}; {}", code_to_str(StatusCode::INSUFFICIENT_STORAGE), err::MSG_DATABASE, &e);
        ApiError::create(507, err::MSG_DATABASE, &e) // 507
    })?;

    let user = result.ok_or_else(|| {
        let msg = format!("user_id: {}", user_id);
        error!("{}-{}; {}", code_to_str(StatusCode::UNAUTHORIZED), err::MSG_UNACCEPTABLE_TOKEN_ID, &msg);
        ApiError::create(401, err::MSG_UNACCEPTABLE_TOKEN_ID, &msg) // 401(d)
    })?;

    if let Some(timer) = timer {
        let s1 = format!("{:.2?}", timer.elapsed());
        #[rustfmt::skip]
        info!("check_token_and_get_user() time: {}, id: {}, nickname: {}", s1, user.id, &user.nickname);
    }
    Ok(user)
}
