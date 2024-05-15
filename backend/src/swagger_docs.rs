use utoipa::{
    openapi::security::{ApiKey, ApiKeyValue, HttpAuthScheme, HttpBuilder, SecurityScheme},
    Modify, OpenApi,
};

use crate::{
    errors,
    streams::{stream_controller, stream_models},
    users::{user_auth_controller, user_controller, user_models, user_registr_controller},
};

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
        user_registr_controller::recovery,
        user_registr_controller::confirm_recovery,
        user_registr_controller::clear_for_expired,
        stream_controller::post_stream,
        stream_controller::put_stream,
    ),
    components(
        schemas(
            errors::AppError,
            user_models::UserRole,
            // user_controller, user_auth_controller, user_registr_controller
            user_models::UserDto,
            // user_auth_controller
            user_models::LoginUserDto, user_models::LoginUserResponseDto, // ::login
            user_models::UserTokensDto, // ::login, ::update_token
            user_models::TokenUserDto, // ::update_token
            // user_controller
            user_models::PasswordUserDto,
            // user_registr_controller
            user_models::RegistrUserDto, user_models::RegistrUserResponseDto, // ::registration
            user_models::RecoveryUserDto, user_models::RecoveryUserResponseDto, // ::recovery
            user_models::RecoveryDataDto, // ::confirm_recovery
            user_models::ClearForExpiredResponseDto, // ::clear_for_expired
            // stream_controller
            stream_models::StreamState, stream_models::StreamInfoDto // ::post_stream, ::put_stream
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
