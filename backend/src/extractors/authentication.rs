use std::rc::Rc;

use actix_web::{dev, error, http, web, FromRequest, HttpMessage};
use futures_util::{
    future::{ready, LocalBoxFuture, Ready},
    FutureExt,
};
use log;

use crate::errors::AppError;
#[cfg(not(feature = "mockdata"))]
use crate::sessions::session_orm::impls::SessionOrmApp;
#[cfg(feature = "mockdata")]
use crate::sessions::session_orm::tests::SessionOrmApp;
use crate::sessions::{config_jwt, session_orm::SessionOrm, tokens::decode_token};
use crate::settings::err;
#[cfg(not(feature = "mockdata"))]
use crate::users::user_orm::impls::UserOrmApp;
#[cfg(feature = "mockdata")]
use crate::users::user_orm::tests::UserOrmApp;
use crate::users::{
    user_models::{User, UserRole},
    user_orm::UserOrm,
};

pub const BEARER: &str = "Bearer ";
// 401 Unauthorized - According to "user_id" in the token, the user was not found.
pub const MSG_UNACCEPTABLE_TOKEN_ID: &str = "unacceptable_token_id";
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
            #[rustfmt::skip]
            None => Err(error::ErrorInternalServerError(AppError::internal_err500(MSG_USER_NOT_RECEIVED_FROM_REQUEST))),
        };
        ready(result)
    }
}

impl std::ops::Deref for Authenticated {
    type Target = User;
    /// Implement the deref method to access the inner User value of Authenticated.
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
    S: dev::Service<
            dev::ServiceRequest,
            Response = dev::ServiceResponse<actix_web::body::BoxBody>,
            Error = actix_web::Error,
        > + 'static,
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

fn jwt_from_header(header_token: &str) -> Result<String, String> {
    const NO_AUTH_HEADER: &str = "No authentication header";
    // eprintln!("jwt_from_header({})", header_token);
    if header_token.len() == 0 {
        return Err(NO_AUTH_HEADER.to_string());
    }
    let auth_header = match std::str::from_utf8(header_token.as_bytes()) {
        Ok(v) => v,
        Err(e) => return Err(format!("{} : {}", NO_AUTH_HEADER, e.to_string())),
    };
    // eprintln!("@auth_header: {}", auth_header.to_owned());
    if !auth_header.starts_with(BEARER) {
        return Err("Invalid authentication header".to_string());
    }
    Ok(auth_header.trim_start_matches(BEARER).to_owned())
}

impl<S> dev::Service<dev::ServiceRequest> for AuthMiddleware<S>
where
    S: dev::Service<
            dev::ServiceRequest,
            Response = dev::ServiceResponse<actix_web::body::BoxBody>,
            Error = actix_web::Error,
        > + 'static,
{
    /// The response type produced by the service.
    type Response = dev::ServiceResponse<actix_web::body::BoxBody>;
    /// The error type that can be produced by the service.
    type Error = actix_web::Error;
    /// The future type representing the asynchronous response.
    type Future = LocalBoxFuture<'static, Result<Self::Response, actix_web::Error>>;

    /// Returns `Ready` when the service is able to process requests.
    fn poll_ready(&self, ctx: &mut core::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
        self.service.poll_ready(ctx)
    }
    /// The future type representing the asynchronous response.
    fn call(&self, req: dev::ServiceRequest) -> Self::Future {
        // Attempt to extract token from cookie or authorization header
        let token = req.cookie("token").map(|c| c.value().to_string()).or_else(|| {
            let header_token = req
                .headers()
                .get(http::header::AUTHORIZATION)
                .map(|h| h.to_str().unwrap().to_string())
                .unwrap_or("".to_string());
            let token2 = jwt_from_header(&header_token)
                .map_err(|e| {
                    log::error!("InvalidToken: {}", e);
                    None::<String>
                })
                .ok();
            token2
        });

        // If token is missing, return unauthorized error
        if token.is_none() {
            log::error!("{}: {}", err::CD_UNAUTHORIZED, err::MSG_MISSING_TOKEN);
            let json_error = AppError::unauthorized401(err::MSG_MISSING_TOKEN);
            return Box::pin(ready(Err(error::ErrorUnauthorized(json_error)))); // 401
        }
        let token = token.unwrap().to_string();

        let config_jwt = req.app_data::<web::Data<config_jwt::ConfigJwt>>().unwrap();
        let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();
        // Decode the token.
        let token_res = decode_token(&token, jwt_secret);

        if let Err(e) = token_res {
            let message = format!("{}: {}", err::MSG_INVALID_OR_EXPIRED_TOKEN, &e);
            log::error!("{}: {}", err::CD_UNAUTHORIZED, &message);
            let json_error = AppError::unauthorized401(&message);
            return Box::pin(ready(Err(error::ErrorUnauthorized(json_error)))); // 401
        }

        let (user_id, num_token) = token_res.unwrap();

        let allowed_roles = self.allowed_roles.clone();
        let srv = Rc::clone(&self.service);

        // Handle user extraction and request processing
        async move {
            let session_orm = req.app_data::<web::Data<SessionOrmApp>>().unwrap();
            // Find a session for a given user.
            let opt_session = session_orm.find_session_by_id(user_id).map_err(|e| {
                log::error!("{}: {}", err::CD_DATABASE, e.to_string());
                let error = AppError::database507(&e.to_string());
                return error::ErrorInsufficientStorage(error); // 507
            })?;

            let session = opt_session.ok_or_else(|| {
                // There is no session for this user.
                let message = format!("{}: user_id: {}", err::MSG_SESSION_NOT_EXIST, user_id);
                log::error!("{}: {}", err::CD_NOT_ACCEPTABLE, &message); // 406
                error::ErrorNotAcceptable(AppError::not_acceptable406(&message))
            })?;
            // Each session contains an additional numeric value.
            let session_num_token = session.num_token.unwrap_or(0);
            // Compare an additional numeric value from the session and from the token.
            if session_num_token != num_token {
                // If they do not match, then this is an error.
                let message = format!("{}: user_id: {}", err::MSG_UNACCEPTABLE_TOKEN_NUM, user_id);
                log::error!("{}: {}", err::CD_UNAUTHORIZED, &message); // 401
                return Err(error::ErrorUnauthorized(AppError::unauthorized401(&message)));
            }

            let user_orm = req.app_data::<web::Data<UserOrmApp>>().unwrap();

            let result = user_orm.find_user_by_id(user_id).map_err(|e| {
                log::error!("{}: {}", err::CD_DATABASE, e.to_string());
                let error = AppError::database507(&e.to_string());
                error::ErrorInsufficientStorage(error) // 507
            })?;

            let user = result.ok_or_else(|| {
                let message = format!("{}: user_id: {}", MSG_UNACCEPTABLE_TOKEN_ID, user_id);
                log::error!("{}: {}", err::CD_UNAUTHORIZED, &message);
                error::ErrorUnauthorized(AppError::unauthorized401(&message)) // 401
            })?;

            // Check if user's role matches the required role
            if allowed_roles.contains(&user.role) {
                // Insert user information into request extensions
                req.extensions_mut().insert::<User>(user);
                // Call the wrapped service to handle the request
                let res = srv.call(req).await?;
                Ok(res)
            } else {
                log::error!("{}: {}", err::CD_FORBIDDEN, err::MSG_ACCESS_DENIED);
                let err_msg = AppError::forbidden403(err::MSG_ACCESS_DENIED);
                Err(error::ErrorForbidden(err_msg)) // 403
            }
        }
        .boxed_local()
    }
}

#[cfg(all(test, feature = "mockdata"))]
mod tests {
    use actix_web::{cookie::Cookie, dev, get, http, test, test::TestRequest, web, App, HttpResponse};

    use crate::sessions::{
        config_jwt, session_models::Session, session_orm::tests::SessionOrmApp, tokens::encode_token,
    };
    use crate::users::{
        user_models::{User, UserRole},
        user_orm::tests::UserOrmApp,
    };

    use super::*;

    #[get("/", wrap = "RequireAuth::allowed_roles(RequireAuth::all_roles())")]
    async fn handler_with_auth() -> HttpResponse {
        HttpResponse::Ok().into()
    }

    #[get("/", wrap = "RequireAuth::allowed_roles(RequireAuth::admin_role())")]
    async fn handler_with_requireonlyadmin() -> HttpResponse {
        HttpResponse::Ok().into()
    }

    fn create_user() -> User {
        let mut user = UserOrmApp::new_user(1, "Oliver_Taylor", "Oliver_Taylor@gmail.com", "passwrN1T1");
        user.role = UserRole::User;
        user
    }
    fn user_with_id(user: User) -> User {
        let user_orm = UserOrmApp::create(&vec![user]);
        user_orm.user_vec.get(0).unwrap().clone()
    }
    fn create_session(user_id: i32, num_token: Option<i32>) -> Session {
        SessionOrmApp::new_session(user_id, num_token)
    }
    fn cfg_jwt() -> config_jwt::ConfigJwt {
        config_jwt::get_test_config()
    }
    fn header_auth(token: &str) -> (http::header::HeaderName, http::header::HeaderValue) {
        let header_value = http::header::HeaderValue::from_str(&format!("{}{}", BEARER, token)).unwrap();
        (http::header::AUTHORIZATION, header_value)
    }
    async fn call_service1(
        cfg_jwt: config_jwt::ConfigJwt,
        vec: (Vec<User>, Vec<Session>),
        factory: impl dev::HttpServiceFactory + 'static,
        request: TestRequest,
    ) -> dev::ServiceResponse {
        let data_config_jwt = web::Data::new(cfg_jwt);

        let data_user_orm = web::Data::new(UserOrmApp::create(&vec.0));
        let data_session_orm = web::Data::new(SessionOrmApp::create(&vec.1));

        let app = test::init_service(
            App::new()
                .app_data(web::Data::clone(&data_config_jwt))
                .app_data(web::Data::clone(&data_user_orm))
                .app_data(web::Data::clone(&data_session_orm))
                .service(factory),
        )
        .await;

        test::call_service(&app, request.to_request()).await
    }
    async fn call_service2(
        config_jwt: config_jwt::ConfigJwt,
        vec: (Vec<User>, Vec<Session>),
        factory: impl dev::HttpServiceFactory + 'static,
        request: TestRequest,
    ) -> Result<dev::ServiceResponse, actix_web::Error> {
        let data_config_jwt = web::Data::new(config_jwt);

        let data_user_orm = web::Data::new(UserOrmApp::create(&vec.0));
        let data_session_orm = web::Data::new(SessionOrmApp::create(&vec.1));

        let app = test::init_service(
            App::new()
                .app_data(web::Data::clone(&data_config_jwt))
                .app_data(web::Data::clone(&data_user_orm))
                .app_data(web::Data::clone(&data_session_orm))
                .service(factory),
        )
        .await;

        test::try_call_service(&app, request.to_request()).await
    }

    #[test]
    async fn test_authentication_middelware_valid_token() {
        let user1: User = user_with_id(create_user());
        let num_token = 1234;
        let session1 = create_session(user1.id, Some(num_token));

        let config_jwt = config_jwt::get_test_config();
        let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();
        let token = encode_token(user1.id, num_token, &jwt_secret, config_jwt.jwt_access).unwrap();

        let request = test::TestRequest::get().insert_header(header_auth(&token));
        let data_c = (vec![user1], vec![session1]);
        let resp = call_service1(config_jwt, data_c, handler_with_auth, request).await;
        assert_eq!(resp.status(), http::StatusCode::OK);
    }

    #[test]
    async fn test_authentication_middelware_valid_token_with_cookie() {
        let user1: User = user_with_id(create_user());
        let num_token = 1234;
        let session1 = create_session(user1.id, Some(num_token));

        let config_jwt = config_jwt::get_test_config();
        let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();
        let token_val = encode_token(user1.id, num_token, &jwt_secret, config_jwt.jwt_access).unwrap();

        let request = test::TestRequest::get().cookie(Cookie::new("token", token_val));
        let data_c = (vec![user1], vec![session1]);
        let resp = call_service1(config_jwt, data_c, handler_with_auth, request).await;
        assert_eq!(resp.status(), http::StatusCode::OK);
    }

    #[test]
    async fn test_authentication_middleware_access_admin_only_endpoint_success() {
        let mut user1a = create_user();
        user1a.role = UserRole::Admin;
        let user1: User = user_with_id(user1a);

        let num_token = 1234;
        let session1 = create_session(user1.id, Some(num_token));

        let config_jwt = config_jwt::get_test_config();
        let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();
        let token = encode_token(user1.id, num_token, &jwt_secret, config_jwt.jwt_access).unwrap();

        let request = test::TestRequest::get().insert_header(header_auth(&token));
        let data_c = (vec![user1], vec![session1]);
        let resp = call_service1(config_jwt, data_c, handler_with_auth, request).await;
        assert_eq!(resp.status(), http::StatusCode::OK);
    }

    #[test]
    async fn test_authentication_middleware_missing_token() {
        let request = test::TestRequest::get();
        let data_c = (vec![], vec![]);
        let result = call_service2(cfg_jwt(), data_c, handler_with_auth, request).await.err();
        let err = result.expect("Service call succeeded, but an error was expected.");

        let actual_status = err.as_response_error().status_code();
        assert_eq!(actual_status, http::StatusCode::UNAUTHORIZED); // 401

        let app_err: AppError = serde_json::from_str(&err.to_string()).expect("Failed to deserialize JSON string");
        assert_eq!(app_err.code, err::CD_UNAUTHORIZED);
        assert_eq!(app_err.message, err::MSG_MISSING_TOKEN);
    }
    #[test]
    async fn test_authentication_middleware_invalid_token() {
        let request = test::TestRequest::get().insert_header(header_auth("invalid_token123"));
        let data_c = (vec![], vec![]);
        let result = call_service2(cfg_jwt(), data_c, handler_with_auth, request).await.err();
        let err = result.expect("Service call succeeded, but an error was expected.");

        let actual_status = err.as_response_error().status_code();
        assert_eq!(actual_status, http::StatusCode::UNAUTHORIZED); // 401

        let app_err: AppError = serde_json::from_str(&err.to_string()).expect("Failed to deserialize JSON string");
        assert_eq!(app_err.code, err::CD_UNAUTHORIZED);
        assert!(app_err.message.starts_with(err::MSG_INVALID_OR_EXPIRED_TOKEN));
    }
    #[test]
    async fn test_authentication_middelware_expired_token() {
        let user1: User = user_with_id(create_user());
        let num_token = 1234;
        let session1 = create_session(user1.id, Some(num_token));

        let config_jwt = config_jwt::get_test_config();
        let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();
        let token = encode_token(user1.id, num_token, &jwt_secret, -config_jwt.jwt_access).unwrap();

        let request = test::TestRequest::get().insert_header(header_auth(&token));
        let data_c = (vec![user1], vec![session1]);
        let result = call_service2(config_jwt, data_c, handler_with_auth, request).await.err();
        let err = result.expect("Service call succeeded, but an error was expected.");

        let actual_status = err.as_response_error().status_code();
        assert_eq!(actual_status, http::StatusCode::UNAUTHORIZED); // 401

        let app_err: AppError = serde_json::from_str(&err.to_string()).expect("Failed to deserialize JSON string");
        assert_eq!(app_err.code, err::CD_UNAUTHORIZED);
        #[rustfmt::skip]
        assert_eq!(app_err.message, format!("{}: ExpiredSignature", err::MSG_INVALID_OR_EXPIRED_TOKEN));
    }
    #[test]
    async fn test_authentication_middelware_valid_token_non_existent_user() {
        let user1: User = user_with_id(create_user());
        let user_id = user1.id + 1;
        let num_token = 1234;
        let session1 = create_session(user_id, Some(num_token));

        let config_jwt = config_jwt::get_test_config();
        let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();
        let token = encode_token(user_id, num_token, &jwt_secret, config_jwt.jwt_access).unwrap();
        // ??
        let request = test::TestRequest::get().insert_header(header_auth(&token));
        let data_c = (vec![user1], vec![session1]);
        let result = call_service2(config_jwt, data_c, handler_with_auth, request).await.err();
        let err = result.expect("Service call succeeded, but an error was expected.");

        let actual_status = err.as_response_error().status_code();
        assert_eq!(actual_status, http::StatusCode::UNAUTHORIZED); // 401

        let app_err: AppError = serde_json::from_str(&err.to_string()).expect("Failed to deserialize JSON string");
        assert_eq!(app_err.code, err::CD_UNAUTHORIZED);
        #[rustfmt::skip]
        assert_eq!(app_err.message, format!("{}: user_id: {}", MSG_UNACCEPTABLE_TOKEN_ID, user_id));
    }
    #[test]
    async fn test_authentication_middelware_valid_token_non_existent_num() {
        let user1: User = user_with_id(create_user());
        let user_id = user1.id;

        let num_token = 1234;
        let session1 = create_session(user_id, Some(num_token));

        let config_jwt = config_jwt::get_test_config();
        let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();
        let token = encode_token(user_id, num_token + 1, &jwt_secret, config_jwt.jwt_access).unwrap();

        let request = test::TestRequest::get().insert_header(header_auth(&token));
        let data_c = (vec![user1], vec![session1]);
        let result = call_service2(config_jwt, data_c, handler_with_auth, request).await.err();
        let err = result.expect("Service call succeeded, but an error was expected.");

        let actual_status = err.as_response_error().status_code();
        assert_eq!(actual_status, http::StatusCode::UNAUTHORIZED); // 401

        let app_err: AppError = serde_json::from_str(&err.to_string()).expect("Failed to deserialize JSON string");
        assert_eq!(app_err.code, err::CD_UNAUTHORIZED);
        #[rustfmt::skip]
        assert_eq!(app_err.message, format!("{}: user_id: {}", err::MSG_UNACCEPTABLE_TOKEN_NUM, user_id));
    }
    #[test]
    async fn test_authentication_middleware_access_admin_only_endpoint_fail() {
        let user1: User = user_with_id(create_user());
        let num_token = 1234;
        let session1 = create_session(user1.id, Some(num_token));

        let config_jwt = config_jwt::get_test_config();
        let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();
        let token = encode_token(user1.id, num_token, &jwt_secret, config_jwt.jwt_access).unwrap();

        let request = test::TestRequest::get().insert_header(header_auth(&token));
        let data_c = (vec![user1], vec![session1]);
        #[rustfmt::skip]
        let result = call_service2(config_jwt, data_c, handler_with_requireonlyadmin, request).await.err();
        let err = result.expect("Service call succeeded, but an error was expected.");

        let actual_status = err.as_response_error().status_code();
        assert_eq!(actual_status, http::StatusCode::FORBIDDEN); // 403

        let app_err: AppError = serde_json::from_str(&err.to_string()).expect("Failed to deserialize JSON string");
        assert_eq!(app_err.code, err::CD_FORBIDDEN);
        assert_eq!(app_err.message, err::MSG_ACCESS_DENIED);
    }
}
