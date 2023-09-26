use std::future::{ready, Ready};

use actix_web::error::ErrorUnauthorized;
use actix_web::{dev::Payload, Error as ActixWebError};
use actix_web::{http, web, FromRequest, HttpMessage, HttpRequest};

use crate::errors::AppError;
use crate::sessions::tokens;

pub struct AuthMiddleware {
    pub user_id: i32,
}

impl FromRequest for AuthMiddleware {
    type Error = ActixWebError;
    type Future = Ready<Result<Self, Self::Error>>;
    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        let token = req.cookie("token").map(|c| c.value().to_string()).or_else(|| {
            req.headers()
                .get(http::header::AUTHORIZATION)
                .map(|h| h.to_str().unwrap().split_at(7).1.to_string())
        });

        if token.is_none() {
            // let json_error = ErrorResponse {
            //     status: "fail".to_string(),
            //     message: "You are not logged in, please provide token".to_string(),
            // };
            let err_msg = "You are not logged in, please provide token";
            let json_error = AppError::new("Authorization", err_msg);
            return ready(Err(ErrorUnauthorized(json_error)));
        }

        let config_jwt = req.app_data::<web::Data<ConfigJwt>>().unwrap();

        let user_id = match tokens::decode_token(&token.unwrap(), config_jwt.jwt_secret.as_bytes())
        {
            Ok(token_claims) => token_claims.sub,
            Err(e) => {
                // let json_error = ErrorResponse { status: "fail".to_string(), message: e.message };
                let json_error = AppError::new("Authorization", &e.to_string());
                return ready(Err(ErrorUnauthorized(json_error)));
            }
        };

        // let user_id = uuid::Uuid::parse_str(user_id.as_str()).unwrap();
        let user_id = user_id.parse::<i32>().unwrap();
        req.extensions_mut().insert::<i32>(user_id.to_owned());

        ready(Ok(AuthMiddleware { user_id }))
    }
}

//#[cfg(test)]
/*mod tests {
    use actix_web::{get, test, App, HttpResponse};
    use sqlx::{Pool, Postgres};

    use crate::{
        db::DBClient,
        utils::{test_utils::get_test_config, token},
    };

    use super::*;

    #[get("/")]
    async fn handler(_: AuthMiddleware) -> HttpResponse {
        HttpResponse::Ok().into()
    }

    #[sqlx::test]
    async fn test_auth_middelware_valid_token(pool: Pool<Postgres>) {
        let user_id = uuid::Uuid::new_v4();
        let db_client = DBClient::new(pool);
        let config = get_test_config();

        let token =
            token::create_token(&user_id.to_string(), config.jwt_secret.as_bytes(), 60).unwrap();

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(AppState {
                    env: config.clone(),
                    db_client,
                }))
                .service(handler),
        )
        .await;

        let req = test::TestRequest::default()
            .insert_header((http::header::AUTHORIZATION, format!("Bearer {}", token)))
            .to_request();

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
                .service(handler),
        )
        .await;

        let req = test::TestRequest::default().to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), http::StatusCode::UNAUTHORIZED);

        let body = test::read_body(resp).await;
        let expected_message = "You are not logged in, please provide token";

        let body_json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        let actual_message = body_json["message"].as_str().unwrap();

        assert_eq!(actual_message, expected_message);
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
                .service(handler),
        )
        .await;

        let req = test::TestRequest::default()
            .insert_header((
                http::header::AUTHORIZATION,
                format!("Bearer {}", "invalid_token"),
            ))
            .to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), http::StatusCode::UNAUTHORIZED);

        let body = test::read_body(resp).await;
        let expected_message = "Authentication token is invalid or expired";

        let body_json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        let actual_message = body_json["message"].as_str().unwrap();

        assert_eq!(actual_message, expected_message);
    }
}*/
