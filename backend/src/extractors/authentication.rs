use std::rc::Rc;

use actix_web::{dev, error, http::StatusCode, web, FromRequest, HttpMessage};
use futures_util::{
    future::{ready, LocalBoxFuture, Ready},
    FutureExt,
};
use log::{debug, error, log_enabled, Level::Info};
use vrb_common::api_error::{code_to_str, ApiError};
use vrb_dbase::db_enums::UserRole;
use vrb_tools::{err, token_coding, token_data};

#[cfg(not(all(test, feature = "mockdata")))]
use crate::profiles::profile_orm::impls::ProfileOrmApp;
#[cfg(all(test, feature = "mockdata"))]
use crate::profiles::profile_orm::tests::ProfileOrmApp;
use crate::profiles::{config_jwt, profile_models::Profile};
use crate::utils::token_verification::check_token_and_get_profile;

// 500 Internal Server Error - Authentication: The entity "user" was not received from the request.
pub const MSG_USER_NOT_RECEIVED_FROM_REQUEST: &str = "user_not_received_from_request";

pub struct Authenticated(Profile);

impl FromRequest for Authenticated {
    type Error = actix_web::Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &actix_web::HttpRequest, _payload: &mut dev::Payload) -> Self::Future {
        let value = req.extensions().get::<Profile>().cloned();
        let result = match value {
            Some(profile) => Ok(Authenticated(profile)),
            None => Err(error::ErrorInternalServerError(ApiError::new(500, MSG_USER_NOT_RECEIVED_FROM_REQUEST))),
        };
        ready(result)
    }
}

impl std::ops::Deref for Authenticated {
    type Target = Profile;
    /// Implement the deref method to access the inner "Profile" value of Authenticated.
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
            let profile_orm = req.app_data::<web::Data<ProfileOrmApp>>().unwrap().get_ref();
            // Check the token for correctness and get the user profile.
            let res_profile = check_token_and_get_profile(user_id, num_token, profile_orm).await;
            if let Err(app_error) = res_profile {
                return Err(app_error.into());
            }
            let profile = res_profile.unwrap();

            if let Some(timer0) = opt_timer0 {
                debug!("timer0: {}, profile.role: {:?}", format!("{:.2?}", timer0.elapsed()), &profile.role);
            }

            // Check if user's role matches the required role
            if allowed_roles.contains(&profile.role) {
                // Insert user profile information into request extensions.
                req.extensions_mut().insert::<Profile>(profile);
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

