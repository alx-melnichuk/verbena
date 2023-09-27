use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::error::{ErrorForbidden, ErrorInternalServerError, ErrorUnauthorized};
use actix_web::{http, web, FromRequest, HttpMessage};
use futures_util::future::{ready, LocalBoxFuture, Ready};
use futures_util::FutureExt;
use log;
use std::rc::Rc;
use std::task::{Context, Poll};

use crate::errors::AppError;
use crate::sessions::{config_jwt::ConfigJwt, tokens};
use crate::users::user_models::{User, /*UserDto,*/ UserRole};
#[cfg(feature = "mockdata")]
use crate::users::user_orm::tests::UserOrmApp;
#[cfg(not(feature = "mockdata"))]
use crate::users::user_orm::UserOrmApp;
use crate::users::user_orm::{UserOrm, CD_DATA_BASE};

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
    pub fn allowed_roles(allowed_roles: Vec<UserRole>) -> Self {
        RequireAuth {
            allowed_roles: Rc::new(allowed_roles),
        }
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
        // Attempt to extract token from cookie or authorization header
        let token = req.cookie("token").map(|c| c.value().to_string()).or_else(|| {
            req.headers()
                .get(http::header::AUTHORIZATION)
                .map(|h| h.to_str().unwrap().split_at(7).1.to_string())
        });
        // If token is missing, return unauthorized error
        if token.is_none() {
            // let json_error = ErrorResponse { status: "fail".to_string(), message: ErrorMessage::TokenNotProvided.to_string()};
            let err_msg = "You are not logged in, please provide token2";
            let json_error = AppError::new("TokenNotProvided", err_msg);
            log::warn!("{}: {}", "TokenNotProvided", err_msg);
            return Box::pin(ready(Err(ErrorUnauthorized(json_error))));
        }

        let config_jwt = req.app_data::<web::Data<ConfigJwt>>().unwrap();
        // Decode token and handle errors
        let user_id = match tokens::decode_token(&token.unwrap(), config_jwt.jwt_secret.as_bytes())
        {
            Ok(token_claims) => token_claims.sub,
            Err(e) => {
                return Box::pin(ready(Err(ErrorUnauthorized(
                    // ErrorResponse { status: "fail".to_string(), message: e.message }
                    AppError::new("Authentication2", &e.to_string()),
                ))));
            }
        };

        // let cloned_app_state = app_state.clone();
        let allowed_roles = self.allowed_roles.clone();
        let srv = Rc::clone(&self.service);

        // Handle user extraction and request processing
        async move {
            // let user_id = uuid::Uuid::parse_str(user_id.as_str()).unwrap();
            let user_id = user_id.parse::<i32>().unwrap();

            // let result = cloned_app_state.db_client.get_user(Some(user_id.clone()), None, None).await
            //     .map_err(|e| ErrorInternalServerError(HttpError::server_error(e.to_string())))?;

            let user_orm = req.app_data::<web::Data<UserOrmApp>>().unwrap();

            let result = user_orm.find_user_by_id(user_id.clone()).map_err(|e| {
                log::warn!("{}: {}", CD_DATA_BASE, e.to_string());
                ErrorInternalServerError(
                    AppError::new(CD_DATA_BASE, &e.to_string()).set_status(500),
                )
                // #-OLD ErrorInternalServerError(AppError::new("Authentication4", &e.to_string()))
            })?;

            let err_msg = "User belonging to this token no longer exists";
            let user = result.ok_or(ErrorUnauthorized(
                // ErrorResponse { status: "fail".to_string(), message: ErrorMessage::UserNoLongerExist.to_string(), }
                AppError::new("UserNoLongerExist", err_msg),
            ))?;

            // Check if user's role matches the required role
            // # if allowed_roles.contains(&user.role) {
            let user_role = allowed_roles.get(0).unwrap();
            if allowed_roles.contains(user_role) {
                // Insert user information into request extensions
                req.extensions_mut().insert::<User>(user);
                // Call the wrapped service to handle the request
                let res = srv.call(req).await?;
                Ok(res)
            } else {
                // let json_error = ErrorResponse {
                //     status: "fail".to_string(),
                //     message: ErrorMessage::PermissionDenied.to_string(),
                // };
                let err_msg = "You are not allowed to perform this action";
                let json_error = AppError::new("Authentication6", err_msg);
                Err(ErrorForbidden(json_error))
            }
        }
        .boxed_local()
    }
}

//#[cfg(test)]
/*mod tests {
    use actix_web::{cookie::Cookie, get, test, App, HttpResponse};
    use sqlx::{Pool, Postgres};

    use crate::{
        db::DBClient,
        extractors::auth::RequireAuth,
        utils::{password, test_utils::get_test_config, token},
    };

    use super::*;

    #[get(
        "/",
        wrap = "RequireAuth::allowed_roles(vec![UserRole::User, UserRole::Moderator, UserRole::Admin])"
    )]
    async fn handler_with_requireauth() -> HttpResponse {
        HttpResponse::Ok().into()
    }

    #[get("/", wrap = "RequireAuth::allowed_roles(vec![UserRole::Admin])")]
    async fn handler_with_requireonlyadmin() -> HttpResponse {
        HttpResponse::Ok().into()
    }

    #[sqlx::test]
    async fn test_auth_middelware_valid_token(pool: Pool<Postgres>) {
        let db_client = DBClient::new(pool);
        let config = get_test_config();

        let hashed_password = password::hash("password123").unwrap();

        let user = db_client.save_user("John", "john@example.com", &hashed_password).await.unwrap();

        let token =
            token::create_token(&user.id.to_string(), config.jwt_secret.as_bytes(), 60).unwrap();

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(AppState {
                    env: config.clone(),
                    db_client,
                }))
                .service(handler_with_requireauth),
        )
        .await;

        let req = test::TestRequest::default()
            .insert_header((http::header::AUTHORIZATION, format!("Bearer {}", token)))
            .to_request();

        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), http::StatusCode::OK);
    }

    #[sqlx::test]
    async fn test_auth_middelware_valid_token_with_cookie(pool: Pool<Postgres>) {
        let db_client = DBClient::new(pool);
        let config = get_test_config();

        let hashed_password = password::hash("password123").unwrap();

        let user = db_client.save_user("John", "john@example.com", &hashed_password).await.unwrap();

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(AppState {
                    env: config.clone(),
                    db_client,
                }))
                .service(handler_with_requireauth),
        )
        .await;

        let token =
            token::create_token(&user.id.to_string(), config.jwt_secret.as_bytes(), 60).unwrap();

        let req = test::TestRequest::default().cookie(Cookie::new("token", token)).to_request();

        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), http::StatusCode::OK);
    }

    #[sqlx::test]
    async fn test_auth_middleware_missing_token(pool: Pool<Postgres>) {
        let db_client = DBClient::new(pool);
        let config = get_test_config();

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(AppState {
                    env: config.clone(),
                    db_client,
                }))
                .service(handler_with_requireauth),
        )
        .await;

        let req = test::TestRequest::default().to_request();
        let result = test::try_call_service(&app, req).await.err();

        match result {
            Some(err) => {
                let expected_status = http::StatusCode::UNAUTHORIZED;
                let actual_status = err.as_response_error().status_code();

                assert_eq!(actual_status, expected_status);

                let err_response: ErrorResponse = serde_json::from_str(&err.to_string())
                    .expect("Failed to deserialize JSON string");
                let expected_message = ErrorMessage::TokenNotProvided.to_string();
                assert_eq!(err_response.message, expected_message);
            }
            None => {
                panic!("Service call succeeded, but an error was expected.");
            }
        }
    }

    #[sqlx::test]
    async fn test_auth_middleware_invalid_token(pool: Pool<Postgres>) {
        let db_client = DBClient::new(pool);
        let config = get_test_config();

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(AppState {
                    env: config.clone(),
                    db_client,
                }))
                .service(handler_with_requireauth),
        )
        .await;

        let req = test::TestRequest::default()
            .insert_header((
                http::header::AUTHORIZATION,
                format!("Bearer {}", "invalid_token"),
            ))
            .to_request();

        let result = test::try_call_service(&app, req).await.err();

        match result {
            Some(err) => {
                let expected_status = http::StatusCode::UNAUTHORIZED;
                let actual_status = err.as_response_error().status_code();

                assert_eq!(actual_status, expected_status);

                let err_response: ErrorResponse = serde_json::from_str(&err.to_string())
                    .expect("Failed to deserialize JSON string");
                let expected_message = ErrorMessage::InvalidToken.to_string();
                assert_eq!(err_response.message, expected_message);
            }
            None => {
                panic!("Service call succeeded, but an error was expected.");
            }
        }
    }

    #[sqlx::test]
    async fn test_auth_middleware_access_admin_only_endpoint_fail(pool: Pool<Postgres>) {
        let db_client = DBClient::new(pool);
        let config = get_test_config();

        let hashed_password = password::hash("password123").unwrap();

        let user = db_client.save_user("John", "john@example.com", &hashed_password).await.unwrap();

        let token =
            token::create_token(&user.id.to_string(), config.jwt_secret.as_bytes(), 60).unwrap();

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(AppState {
                    env: config.clone(),
                    db_client,
                }))
                .service(handler_with_requireonlyadmin),
        )
        .await;

        let req = test::TestRequest::default()
            .insert_header((http::header::AUTHORIZATION, format!("Bearer {}", token)))
            .to_request();

        let result = test::try_call_service(&app, req).await.err();

        match result {
            Some(err) => {
                let expected_status = http::StatusCode::FORBIDDEN;
                let actual_status = err.as_response_error().status_code();

                assert_eq!(actual_status, expected_status);

                let err_response: ErrorResponse = serde_json::from_str(&err.to_string())
                    .expect("Failed to deserialize JSON string");
                let expected_message = ErrorMessage::PermissionDenied.to_string();
                assert_eq!(err_response.message, expected_message);
            }
            None => {
                panic!("Service call succeeded, but an error was expected.");
            }
        }
    }

    #[sqlx::test]
    async fn test_auth_middleware_access_admin_only_endpoint_success(pool: Pool<Postgres>) {
        let db_client = DBClient::new(pool.clone());
        let config = get_test_config();

        let hashed_password = password::hash("password123").unwrap();
        let user = db_client
            .save_admin_user("John Doe", "johndoe@gmail.com", &hashed_password)
            .await
            .unwrap();

        let token =
            token::create_token(&user.id.to_string(), config.jwt_secret.as_bytes(), 60).unwrap();

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(AppState {
                    env: config.clone(),
                    db_client,
                }))
                .service(handler_with_requireonlyadmin),
        )
        .await;

        let req = test::TestRequest::default()
            .insert_header((http::header::AUTHORIZATION, format!("Bearer {}", token)))
            .to_request();

        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), http::StatusCode::OK);
    }
}*/
