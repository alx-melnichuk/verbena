use utoipa::{
    openapi::security::{ApiKey, ApiKeyValue, HttpAuthScheme, HttpBuilder, SecurityScheme},
    Modify, OpenApi,
};

use crate::{errors, users::{user_models, user_auth_controller, user_controller, user_registr_controller}};

#[derive(OpenApi)]
#[openapi(
    paths(
        user_auth_controller::login,
        user_auth_controller::logout,
        user_auth_controller::update_token,
        user_controller::get_user_by_email,
        user_controller::get_user_by_nickname,
        user_controller::get_user_by_id,
        user_controller::put_user,
        user_controller::delete_user,
        user_controller::get_user_current,
        user_controller::put_user_current,
        user_controller::delete_user_current,
        user_registr_controller::registration,
        user_registr_controller::confirm_registration,
        // user_registr_controller::recovery,
        // user_registr_controller::confirm_recovery,
    ),
    components(
        schemas(
            errors::AppError,
            user_models::UserRole,
            // user_controller, user_auth_controller, user_registr_controller
            user_models::UserDto,
            // user_auth_controller
            user_models::LoginUserDto, user_models::LoginUserResponseDto, 
            // user_auth_controller::login, user_auth_controller::update_token
            user_models::UserTokensDto,
            // user_auth_controller::update_token
            user_models::TokenUserDto,
            // user_controller
            user_models::PasswordUserDto,
            // user_registr_controller::registration
            user_models::RegistrUserDto, user_models::RegistrUserResponseDto,
        )
    ),
    tags(
        (name = "user_auth_controller", description = "User session management endpoints."),
        (name = "user_controller", description = "User information management endpoints."),
        (name = "user_registr_controller", description = "User registration management endpoints."),
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
