use utoipa::{
    openapi::security::{ApiKey, ApiKeyValue, HttpAuthScheme, HttpBuilder, SecurityScheme},
    Modify, OpenApi,
};

use crate::{errors, user_auth_controller, user_controller, users::user_models};

#[derive(OpenApi)]
#[openapi(
    paths(
        user_auth_controller::login,
        user_controller::get_user_by_email,
        user_controller::get_user_by_nickname,
        user_controller::get_user_by_id,
        user_controller::put_user,
        user_controller::delete_user,
        user_controller::get_user_current,
        user_controller::put_user_current,
        user_controller::delete_user_current,
    ),
    components(
        schemas(
            errors::AppError,
            user_models::UserRole,
            // user_controller, user_auth_controller
            user_models::UserDto,
            // user_controller
            user_models::PasswordUserDto,
            // user_auth_controller::login
            user_models::LoginUserDto, user_models::LoginUserResponseDto, user_models::UserTokensDto,
        )
    ),
    tags(
        (name = "user_auth_controller", description = "User session management endpoints."),
        (name = "user_controller", description = "User information management endpoints.")
    ),
    modifiers(&SecurityAddon)
)]
pub struct ApiDoc;

// println!("{}", ApiDoc::openapi().to_pretty_json().unwrap());

pub struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        // NOTE: we can unwrap safely since there already is components registered.
        let components = openapi.components.as_mut().unwrap();
        components.add_security_scheme(
            "api_key",
            SecurityScheme::ApiKey(ApiKey::Header(ApiKeyValue::new("todo_apikey"))),
        );

        components.add_security_scheme(
            "bearer_auth",
            SecurityScheme::Http(HttpBuilder::new().scheme(HttpAuthScheme::Bearer).bearer_format("JWT").build()),
            // SecurityScheme::Http(Http::new(HttpAuthScheme::Bearer)),
        );
    }
}
