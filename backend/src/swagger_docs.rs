use utoipa::{
    openapi::security::{/*ApiKey, ApiKeyValue,*/ HttpAuthScheme, HttpBuilder, SecurityScheme},
    Modify, OpenApi,
};
use vrb_chats::{chat_message_controller, chat_message_models, chat_ws_controller};
use vrb_common::api_error;
use vrb_dbase::db_enums;
use vrb_streams::{stream_controller, stream_models};

use crate::profiles::{profile_auth_controller, profile_controller, profile_models, profile_registr_controller};

#[derive(OpenApi)]
#[openapi(
    paths(
        chat_message_controller::get_chat_message,
        chat_message_controller::post_chat_message,
        chat_message_controller::put_chat_message,
        chat_message_controller::delete_chat_message,
        chat_message_controller::get_blocked_users,
        chat_message_controller::post_blocked_user,
        chat_message_controller::delete_blocked_user,
        chat_ws_controller::get_ws_chat,
        profile_controller::uniqueness_check,
        profile_controller::get_profile_by_id,
        profile_controller::get_profile_current,
        profile_controller::get_profile_config,
        profile_controller::put_profile,
        profile_controller::put_profile_new_password,
        profile_controller::delete_profile,
        profile_controller::delete_profile_current,
        profile_auth_controller::login,
        profile_auth_controller::logout,
        profile_auth_controller::update_token,
        profile_registr_controller::registration,
        profile_registr_controller::confirm_registration,
        profile_registr_controller::recovery,
        profile_registr_controller::confirm_recovery,
        profile_registr_controller::clear_for_expired,
        stream_controller::get_streams,
        stream_controller::get_stream_by_id,
        stream_controller::get_stream_config,
        stream_controller::get_streams_events,
        stream_controller::get_streams_period,
        stream_controller::post_stream,
        stream_controller::put_stream,
        stream_controller::put_toggle_state,
        stream_controller::delete_stream,
    ),
    components(
        schemas(
            // chat_message_controller
            chat_message_models::SearchChatMessageDto, // ::get_chat_message
            chat_message_models::ChatMessageDto, // ::get_chat_message
            chat_message_models::CreateChatMessageDto, // ::post_chat_message
            chat_message_models::ModifyChatMessageDto, // ::put_chat_message
            chat_message_models::BlockedUserDto, // ::get_blocked_users
            chat_message_models::CreateBlockedUserDto, // ::post_blocked_user
            chat_message_models::DeleteBlockedUserDto, // ::delete_blocked_user
            api_error::ApiError,
            // user model
            db_enums::UserRole,
            // profile_controller
            profile_models::UniquenessProfileDto, // ::uniqueness_check
            profile_models::UniquenessProfileResponseDto, // ::uniqueness_check
            profile_models::ProfileConfigDto, // ::get_profile_config
            // profile_controller
            profile_models::ProfileDto, // ::put_profile
            profile_models::ModifyProfileDto, // ::put_stream
            profile_models::NewPasswordProfileDto, // ::put_profile_new_password,
            // profile_auth_controller
            profile_models::LoginProfileDto, profile_models::LoginProfileResponseDto, // ::login
            profile_models::ProfileTokensDto, // ::login, ::update_token
            profile_models::TokenDto, // ::update_token
            // profile_registr_controller
            profile_models::RegistrProfileDto, profile_models::RegistrProfileResponseDto, // ::registration
            // profile_models::ProfileDto, // ::confirm_registration
            profile_models::RecoveryProfileDto, profile_models::RecoveryProfileResponseDto, // ::recovery
            profile_models::RecoveryDataDto, // profile_models::ProfileDto // ::confirm_recovery
            profile_models::ClearForExpiredResponseDto, // ::clear_for_expired
            // stream_controller
            stream_models::StreamInfoDto,
            stream_models::SearchStreamInfoDto, // ::get_streams
            stream_models::StreamInfoPageDto, // ::get_streams
            stream_models::SearchStreamEventDto, // ::get_streams_events
            stream_models::StreamEventDto, stream_models::StreamEventPageDto, // ::get_streams_events
            stream_models::SearchStreamPeriodDto, // ::get_streams_period
            // stream_controller
            db_enums::StreamState,
            stream_models::CreateStreamInfoDto, // ::post_stream
            stream_models::ModifyStreamInfoDto, // ::put_stream
            stream_models::ToggleStreamStateDto, // ::put_toggle_state
        )
    ),
    tags(
        (name = "chat_message_controller", description = "Managing data for chat work (endpoints)."),
        (name = "chat_ws_controller", description = "Manage messages in chat (Endpoints)."),
        (name = "profile_controller", description = "Managing user profile information (Endpoints)."),
        (name = "profile_auth_controller", description = "User authorization management (Endpoints)."),
        (name = "profile_registr_controller", description = "User registration management (Endpoints)."),
        (name = "stream_controller", description = "Stream management. (Endpoints)."),
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
        // components.add_security_scheme(
        //     "api_key",
        //     SecurityScheme::ApiKey(ApiKey::Header(ApiKeyValue::new("todo_apikey"))),
        // );

        components.add_security_scheme(
            "bearer_auth",
            SecurityScheme::Http(HttpBuilder::new().scheme(HttpAuthScheme::Bearer).bearer_format("JWT").build()),
            // SecurityScheme::Http(Http::new(HttpAuthScheme::Bearer)),
        );
    }
}
