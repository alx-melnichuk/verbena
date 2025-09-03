use utoipa::{
    openapi::security::{/*ApiKey, ApiKeyValue,*/ HttpAuthScheme, HttpBuilder, SecurityScheme},
    Modify, OpenApi,
};
use vrb_authent::{
    user_authent_controller, user_authent_models, user_recovery_controller, user_recovery_models, user_registr_controller,
    user_registr_models,
};
use vrb_chats::{chat_message_controller, chat_message_models, chat_ws_controller};
use vrb_common::api_error;
use vrb_dbase::{enm_stream_state, enm_user_role};
use vrb_profiles::{profile_controller, profile_models};
use vrb_streams::{stream_controller, stream_models};

#[derive(OpenApi)]
#[openapi(
    paths(
        user_authent_controller::users_uniqueness,
        user_authent_controller::logout,
        user_authent_controller::update_token,
        user_registr_controller::registration,
        user_registr_controller::confirm_registration,
        user_registr_controller::registration_clear_for_expired,
        user_recovery_controller::recovery,
        user_recovery_controller::confirm_recovery,
        user_recovery_controller::recovery_clear_for_expired,
        //
        profile_controller::get_profile_by_id,
        profile_controller::get_profile_current,
        profile_controller::get_profile_config,
        profile_controller::put_profile,
        profile_controller::put_profile_new_password,
        profile_controller::delete_profile,
        profile_controller::delete_profile_current,
        //
        stream_controller::get_streams,
        stream_controller::get_stream_by_id,
        stream_controller::get_stream_config,
        stream_controller::get_streams_events,
        stream_controller::get_streams_period,
        stream_controller::post_stream,
        stream_controller::put_stream,
        stream_controller::put_toggle_state,
        stream_controller::delete_stream,
        //
        chat_message_controller::get_chat_message,
        chat_message_controller::post_chat_message,
        chat_message_controller::put_chat_message,
        chat_message_controller::delete_chat_message,
        chat_message_controller::get_blocked_users,
        chat_message_controller::post_blocked_user,
        chat_message_controller::delete_blocked_user,
        chat_ws_controller::get_ws_chat,
    ),
    components(
        schemas(
            api_error::ApiError,
            // vrb_dbase
            enm_user_role::UserRole,
            // user_authent_controller
            user_authent_models::UserUniquenessDto, // ::users_uniqueness
            user_authent_models::UserUniquenessResponseDto, // ::users_uniqueness
            user_authent_models::TokenUserDto, // ::update_token
            user_authent_models::TokenUserResponseDto, // ::update_token
            // user_registr_controller
            user_registr_models::RegistrUserDto, // ::registration
            user_registr_models::RegistrUserResponseDto, // ::registration
            user_registr_models::ConfirmRegistrUserResponseDto, // ::confirm_registration
            user_registr_models::RegistrationClearForExpiredResponseDto, // ::registration_clear_for_expired
            // user_recovery_controller
            user_recovery_models::RecoveryUserDto, // ::recovery
            user_recovery_models::RecoveryUserResponseDto, // ::recovery
            user_recovery_models::ConfirmRecoveryUserResponseDto, // ::confirm_recovery
            user_recovery_models::RecoveryClearForExpiredResponseDto, // ::recovery_clear_for_expired

            // profile_controller
            profile_models::ProfileConfigDto, // ::get_profile_config
            // profile_controller
            profile_models::ProfileDto, // ::put_profile
            profile_models::ModifyProfileDto, // ::put_stream
            profile_models::NewPasswordProfileDto, // ::put_profile_new_password,

            // stream_controller
            enm_stream_state::StreamState,
            stream_models::StreamInfoDto,
            stream_models::SearchStreamInfoDto, // ::get_streams
            stream_models::StreamInfoPageDto, // ::get_streams
            stream_models::SearchStreamEventDto, // ::get_streams_events
            stream_models::StreamEventDto, stream_models::StreamEventPageDto, // ::get_streams_events
            stream_models::SearchStreamPeriodDto, // ::get_streams_period
            stream_models::CreateStreamInfoDto, // ::post_stream
            stream_models::ModifyStreamInfoDto, // ::put_stream
            stream_models::ToggleStreamStateDto, // ::put_toggle_state

            // chat_message_controller
            chat_message_models::SearchChatMessageDto, // ::get_chat_message
            chat_message_models::ChatMessageDto, // ::get_chat_message
            chat_message_models::CreateChatMessageDto, // ::post_chat_message
            chat_message_models::ModifyChatMessageDto, // ::put_chat_message
            chat_message_models::BlockedUserDto, // ::get_blocked_users
            chat_message_models::CreateBlockedUserDto, // ::post_blocked_user
            chat_message_models::DeleteBlockedUserDto, // ::delete_blocked_user
        )
    ),
    tags(
        (name = "user_authent_controller", description = "User authorization management (Endpoints)."),
        (name = "user_registr_controller", description = "User registration management (Endpoints)."),
        (name = "user_recovery_controller", description = "Manage user password recovery (endpoints)."),
        (name = "profile_controller", description = "Managing user profile information (Endpoints)."),
        (name = "stream_controller", description = "Stream management. (Endpoints)."),
        (name = "chat_message_controller", description = "Managing data for chat work (endpoints)."),
        (name = "chat_ws_controller", description = "Manage messages in chat (Endpoints)."),
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
