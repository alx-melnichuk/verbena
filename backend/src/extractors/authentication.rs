use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::error::{ErrorForbidden, ErrorInternalServerError, ErrorUnauthorized};
use actix_web::{http, web, FromRequest, HttpMessage};
use futures_util::future::{ready, LocalBoxFuture, Ready};
use futures_util::FutureExt;
use log;
use std::rc::Rc;
use std::task::{Context, Poll};

use crate::errors::AppError;
#[cfg(not(feature = "mockdata"))]
use crate::sessions::session_orm::inst::SessionOrmApp;
#[cfg(feature = "mockdata")]
use crate::sessions::session_orm::tests::SessionOrmApp;
use crate::sessions::{session_orm::SessionOrm, tools_token, config_jwt::ConfigJwt};
use crate::users::user_models::{User, UserRole};
#[cfg(feature = "mockdata")]
use crate::users::user_orm::tests::UserOrmApp;
#[cfg(not(feature = "mockdata"))]
use crate::users::user_orm::inst::UserOrmApp;
use crate::users::user_orm::UserOrm;
use crate::utils::err;

pub struct Authenticated(User);

impl FromRequest for Authenticated {
    type Error = actix_web::Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(
        req: &actix_web::HttpRequest,
        _payload: &mut actix_web::dev::Payload,
    ) -> Self::Future {
        let value = req.extensions().get::<User>().cloned();
        let result = match value {
            Some(user) => Ok(Authenticated(user)),
            None => Err(ErrorInternalServerError(AppError::new(
                "Authentication",
                "Authentication Error",
            ))),
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

impl<S> Transform<S, ServiceRequest> for RequireAuth
where
    S: Service<
            ServiceRequest,
            Response = ServiceResponse<actix_web::body::BoxBody>,
            Error = actix_web::Error,
        > + 'static,
{
    /// The response type produced by the service.
    type Response = ServiceResponse<actix_web::body::BoxBody>;
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

impl<S> Service<ServiceRequest> for AuthMiddleware<S>
where
    S: Service<
            ServiceRequest,
            Response = ServiceResponse<actix_web::body::BoxBody>,
            Error = actix_web::Error,
        > + 'static,
{
    /// The response type produced by the service.
    type Response = ServiceResponse<actix_web::body::BoxBody>;
    /// The error type that can be produced by the service.
    type Error = actix_web::Error;
    /// The future type representing the asynchronous response.
    type Future = LocalBoxFuture<'static, Result<Self::Response, actix_web::Error>>;

    /// Returns `Ready` when the service is able to process requests.
    fn poll_ready(&self, ctx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(ctx)
    }
    /// The future type representing the asynchronous response.
    fn call(&self, req: ServiceRequest) -> Self::Future {
        // v-- new fn

        // Attempt to extract token from cookie or authorization header
        let token = req.cookie("token").map(|c| c.value().to_string()).or_else(|| {
            req.headers()
                .get(http::header::AUTHORIZATION)
                .map(|h| h.to_str().unwrap().to_string())
        });
        // If token is missing, return unauthorized error
        if token.is_none() {
            log::debug!("{}: {}", err::CD_MISSING_TOKEN, err::MSG_MISSING_TOKEN);
            let json_error =
                AppError::new(err::CD_MISSING_TOKEN, err::MSG_MISSING_TOKEN).set_status(401);
            return Box::pin(ready(Err(ErrorUnauthorized(json_error))));
        }

        let config_jwt = req.app_data::<web::Data<ConfigJwt>>().unwrap();

        let token = token.unwrap();
        let jwt_secret: &[u8] = config_jwt.jwt_secret.as_bytes();

        let token_res = tools_token::parse_token(&token, jwt_secret)
        .map_err(|e| {
            log::debug!("{}: {}", e.code, e.message);
            return Box::pin(ready(ErrorForbidden(e)));
        });
        let (user_id, num_token) = token_res.unwrap();
        
        let allowed_roles = self.allowed_roles.clone();
        let srv = Rc::clone(&self.service);

        // Handle user extraction and request processing
        async move {
            let session_orm = req.app_data::<web::Data<SessionOrmApp>>().unwrap();
            
            let session_opt = session_orm.find_session_by_id(user_id).map_err(|e| {
                log::debug!("{}: {}", err::CD_DATABASE, e.to_string());
                let json_error = AppError::new(err::CD_DATABASE, &e.to_string()).set_status(500);
                return ErrorInternalServerError(json_error);
            })?;
            
            let session = session_opt.ok_or_else(|| {
                eprintln!("$^session with {} not found", user_id.clone());
                #[rustfmt::skip]
                log::debug!("{}: session with {} not found", err::CD_UNACCEPTABLE_TOKEN, user_id);
                let json_error = AppError::new(err::CD_UNACCEPTABLE_TOKEN, err::MSG_UNACCEPTABLE_TOKEN)
                    .set_status(403); // ?!?
                ErrorForbidden(json_error)
            })?;

            let session_num_token = session.num_token.unwrap_or(0);
            if session_num_token != num_token {
                eprintln!("$^session_num_token != num_token");
                log::debug!("{}: session with {} not found", err::CD_UNACCEPTABLE_TOKEN, user_id);
                let json_error = AppError::new(err::CD_UNACCEPTABLE_TOKEN, err::MSG_UNACCEPTABLE_TOKEN)
                    .set_status(403); // ?!?
                return Err(ErrorForbidden(json_error));
            }

            eprintln!("$^user_id: {}, num_token: {}", user_id.clone(), num_token);

            let user_orm = req.app_data::<web::Data<UserOrmApp>>().unwrap();
            
            let result = user_orm.find_user_by_id(user_id.clone()).map_err(|e| {
                log::debug!("{}: {}", err::CD_DATABASE, e.to_string());
                ErrorInternalServerError(AppError::new(err::CD_DATABASE, &e.to_string()))
            })?;

            let user = result.ok_or_else(|| {
                log::debug!("{}: user with {} not found", err::CD_UNACCEPTABLE_TOKEN, user_id.clone());
                let json_error = AppError::new(err::CD_UNACCEPTABLE_TOKEN, err::MSG_UNACCEPTABLE_TOKEN)
                    .set_status(403); // ?!?
                ErrorForbidden(json_error)
            })?;

            // Check if user's role matches the required role
            if allowed_roles.contains(&user.role) {
                // Insert user information into request extensions
                req.extensions_mut().insert::<User>(user);
                // Call the wrapped service to handle the request
                let res = srv.call(req).await?;
                Ok(res)
            } else {
                #[rustfmt::skip]
                log::debug!("{}: {}", err::CD_PERMISSION_DENIED,err::MSG_PERMISSION_DENIED);
                #[rustfmt::skip]
                let json_error = AppError::new(err::CD_PERMISSION_DENIED, err::MSG_PERMISSION_DENIED)
                    .set_status(403);
                Err(ErrorForbidden(json_error))
            }
        }
        .boxed_local()
    }
}

#[cfg(all(test, feature = "mockdata"))]
mod tests {
    use actix_web::{cookie::Cookie, get, test, App, HttpResponse};

    use crate::sessions::{config_jwt, tokens};
    use crate::users::{user_models, user_orm::tests::UserOrmApp};
    use crate::utils::err;

    use super::*;

    #[get("/", wrap = "RequireAuth::allowed_roles(RequireAuth::all_roles())")]
    async fn handler_with_requireauth() -> HttpResponse {
        HttpResponse::Ok().into()
    }

    #[get("/", wrap = "RequireAuth::allowed_roles(RequireAuth::admin_role())")]
    async fn handler_with_requireonlyadmin() -> HttpResponse {
        HttpResponse::Ok().into()
    }

    fn create_user() -> user_models::User {
        let mut user = UserOrmApp::new_user(
            1001,
            "Oliver_Taylor",
            "Oliver_Taylor@gmail.com",
            "passwordD1T1",
        );
        user.role = UserRole::User;
        user
    }

    #[test]
    async fn test_auth_middelware_valid_token() {
        let user1: user_models::User = create_user();
        let user_id = user1.id.to_string();

        let config_jwt = config_jwt::get_test_config();
        let token = tokens::create_token(&user_id, config_jwt.jwt_secret.as_bytes(), 60).unwrap();

        let data_config_jwt = web::Data::new(config_jwt.clone());
        let data_user_orm = web::Data::new(UserOrmApp::create(vec![user1]));

        let app = test::init_service(
            App::new()
                .app_data(web::Data::clone(&data_config_jwt))
                .app_data(web::Data::clone(&data_user_orm))
                .service(handler_with_requireauth),
        )
        .await;
        let req = test::TestRequest::get()
            .insert_header((http::header::AUTHORIZATION, format!("Bearer {}", token)))
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), http::StatusCode::OK);
    }

    #[test]
    async fn test_auth_middelware_valid_token_with_cookie() {
        let user1: user_models::User = create_user();
        let user_id = user1.id.to_string();

        let config_jwt = config_jwt::get_test_config();
        let token = tokens::create_token(&user_id, config_jwt.jwt_secret.as_bytes(), 60).unwrap();

        let data_config_jwt = web::Data::new(config_jwt.clone());
        let data_user_orm = web::Data::new(UserOrmApp::create(vec![user1]));

        let app = test::init_service(
            App::new()
                .app_data(web::Data::clone(&data_config_jwt))
                .app_data(web::Data::clone(&data_user_orm))
                .service(handler_with_requireauth),
        )
        .await;

        let req = test::TestRequest::get().cookie(Cookie::new("token", token)).to_request();

        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), http::StatusCode::OK);
    }

    #[test]
    async fn test_auth_middleware_missing_token() {
        let config_jwt = config_jwt::get_test_config();

        let data_config_jwt = web::Data::new(config_jwt.clone());
        let data_user_orm = web::Data::new(UserOrmApp::create(vec![create_user()]));

        let app = test::init_service(
            App::new()
                .app_data(web::Data::clone(&data_config_jwt))
                .app_data(web::Data::clone(&data_user_orm))
                .service(handler_with_requireauth),
        )
        .await;

        let req = test::TestRequest::get().to_request();

        let result = test::try_call_service(&app, req).await.err();

        let err = result.expect("Service call succeeded, but an error was expected.");

        let actual_status = err.as_response_error().status_code();
        assert_eq!(actual_status, http::StatusCode::UNAUTHORIZED);

        let app_err: AppError =
            serde_json::from_str(&err.to_string()).expect("Failed to deserialize JSON string");
        assert_eq!(app_err.code, err::CD_MISSING_TOKEN);
        assert_eq!(app_err.message, err::MSG_MISSING_TOKEN);
    }

    #[test]
    async fn test_auth_middleware_invalid_token() {
        let config_jwt = config_jwt::get_test_config();

        let data_config_jwt = web::Data::new(config_jwt.clone());
        let data_user_orm = web::Data::new(UserOrmApp::create(vec![create_user()]));

        let app = test::init_service(
            App::new()
                .app_data(web::Data::clone(&data_config_jwt))
                .app_data(web::Data::clone(&data_user_orm))
                .service(handler_with_requireauth),
        )
        .await;

        let req = test::TestRequest::get()
            .insert_header((
                http::header::AUTHORIZATION,
                format!("Bearer {}", "invalid_token"),
            ))
            .to_request();

        let result = test::try_call_service(&app, req).await.err();

        let err = result.expect("Service call succeeded, but an error was expected.");

        let actual_status = err.as_response_error().status_code();
        assert_eq!(actual_status, http::StatusCode::FORBIDDEN);

        let app_err: AppError =
            serde_json::from_str(&err.to_string()).expect("Failed to deserialize JSON string");
        assert_eq!(app_err.code, err::CD_INVALID_TOKEN);
        assert_eq!(app_err.message, err::MSG_INVALID_TOKEN);
    }

    #[test]
    async fn test_auth_middelware_expired_token() {
        let user1: user_models::User = create_user();
        let user_id: String = user1.id.to_string();

        let config_jwt = config_jwt::get_test_config();

        let expired_token =
            tokens::create_token(&user_id, config_jwt.jwt_secret.as_bytes(), -60).unwrap();

        let data_config_jwt = web::Data::new(config_jwt.clone());
        let data_user_orm = web::Data::new(UserOrmApp::create(vec![user1]));

        let app = test::init_service(
            App::new()
                .app_data(web::Data::clone(&data_config_jwt))
                .app_data(web::Data::clone(&data_user_orm))
                .service(handler_with_requireauth),
        )
        .await;
        let req = test::TestRequest::get()
            .insert_header((
                http::header::AUTHORIZATION,
                format!("Bearer {}", expired_token),
            ))
            .to_request();

        let result = test::try_call_service(&app, req).await.err();

        let err = result.expect("Service call succeeded, but an error was expected.");

        let actual_status = err.as_response_error().status_code();
        assert_eq!(actual_status, http::StatusCode::FORBIDDEN);

        let app_err: AppError =
            serde_json::from_str(&err.to_string()).expect("Failed to deserialize JSON string");
        assert_eq!(app_err.code, err::CD_INVALID_TOKEN);
        assert_eq!(app_err.message, err::MSG_INVALID_TOKEN);
    }

    #[test]
    async fn test_auth_middelware_valid_token_non_existent_user() {
        let user1: user_models::User = create_user();
        let bad_id = format!("{}9999", user1.id);

        let config_jwt = config_jwt::get_test_config();
        let token = tokens::create_token(&bad_id, config_jwt.jwt_secret.as_bytes(), 60).unwrap();

        let data_config_jwt = web::Data::new(config_jwt.clone());
        let data_user_orm = web::Data::new(UserOrmApp::create(vec![user1]));

        let app = test::init_service(
            App::new()
                .app_data(web::Data::clone(&data_config_jwt))
                .app_data(web::Data::clone(&data_user_orm))
                .service(handler_with_requireauth),
        )
        .await;

        let req = test::TestRequest::get()
            .insert_header((http::header::AUTHORIZATION, format!("Bearer {}", token)))
            .to_request();

        let result = test::try_call_service(&app, req).await.err();

        let err = result.expect("Service call succeeded, but an error was expected.");

        let actual_status = err.as_response_error().status_code();
        assert_eq!(actual_status, http::StatusCode::FORBIDDEN);

        let app_err: AppError =
            serde_json::from_str(&err.to_string()).expect("Failed to deserialize JSON string");
        assert_eq!(app_err.code, err::CD_UNACCEPTABLE_TOKEN);
        assert_eq!(app_err.message, err::MSG_UNACCEPTABLE_TOKEN);
    }

    #[test]
    async fn test_auth_middleware_access_admin_only_endpoint_fail() {
        let user1: user_models::User = create_user();
        let user_id = user1.id.to_string();

        let config_jwt = config_jwt::get_test_config();
        let token = tokens::create_token(&user_id, config_jwt.jwt_secret.as_bytes(), 60).unwrap();

        let data_config_jwt = web::Data::new(config_jwt.clone());
        let data_user_orm = web::Data::new(UserOrmApp::create(vec![user1]));

        let app = test::init_service(
            App::new()
                .app_data(web::Data::clone(&data_config_jwt))
                .app_data(web::Data::clone(&data_user_orm))
                .service(handler_with_requireonlyadmin),
        )
        .await;

        let req = test::TestRequest::get()
            .insert_header((http::header::AUTHORIZATION, format!("Bearer {}", token)))
            .to_request();

        let result = test::try_call_service(&app, req).await.err();

        let err = result.expect("Service call succeeded, but an error was expected.");

        let actual_status = err.as_response_error().status_code();
        assert_eq!(actual_status, http::StatusCode::FORBIDDEN);

        let app_err: AppError =
            serde_json::from_str(&err.to_string()).expect("Failed to deserialize JSON string");
        assert_eq!(app_err.code, err::CD_PERMISSION_DENIED);
        assert_eq!(app_err.message, err::MSG_PERMISSION_DENIED);
    }

    #[test]
    async fn test_auth_middleware_access_admin_only_endpoint_success() {
        let mut user1: user_models::User = create_user();
        user1.role = UserRole::Admin;
        let user_id = user1.id.to_string();

        let config_jwt = config_jwt::get_test_config();
        let token = tokens::create_token(&user_id, config_jwt.jwt_secret.as_bytes(), 60).unwrap();

        let data_config_jwt = web::Data::new(config_jwt.clone());
        let data_user_orm = web::Data::new(UserOrmApp::create(vec![user1]));

        let app = test::init_service(
            App::new()
                .app_data(web::Data::clone(&data_config_jwt))
                .app_data(web::Data::clone(&data_user_orm))
                .service(handler_with_requireonlyadmin),
        )
        .await;

        let req = test::TestRequest::get()
            .insert_header((http::header::AUTHORIZATION, format!("Bearer {}", token)))
            .to_request();

        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), http::StatusCode::OK);
    }
}
