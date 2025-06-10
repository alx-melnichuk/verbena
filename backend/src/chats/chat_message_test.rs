#[cfg(all(test, feature = "mockdata"))]
mod tests {
    use actix_web::{
        self, body, dev,
        http::{
            header::{HeaderValue, CONTENT_TYPE},
            StatusCode,
        },
        test, App,
    };
    // use chrono::{DateTime, Datelike, Duration, Local, SecondsFormat, TimeZone, Timelike, Utc};
    use serde_json::json;

    use crate::chats::{
        chat_message_controller::{
            post_chat_message,
            tests::{configure_chat_message, get_cfg_data, header_auth},
        },
        chat_message_models::{self, tests::ChatMessageTest, CreateChatMessageDto},
    };
    use crate::errors::AppError;
    use crate::settings::err;

    const MSG_CONTENT_TYPE_ERROR: &str = "Content type error";
    const MSG_FAILED_DESER: &str = "Failed to deserialize response from JSON.";

    // GET /ws
    //.service(get_ws_chat)
    // GET /api/chat_messages
    //.service(get_chat_message)

    fn check_app_err(app_err_vec: Vec<AppError>, code: &str, msgs: &[&str]) {
        assert_eq!(app_err_vec.len(), msgs.len());
        for (idx, msg) in msgs.iter().enumerate() {
            let app_err = app_err_vec.get(idx).unwrap();
            assert_eq!(app_err.code, code);
            assert_eq!(app_err.message, msg.to_string());
        }
    }

    // ** post_chat_message **

    #[actix_web::test]
    async fn test_post_chat_message_no_form() {
        let (cfg_c, data_c, token) = get_cfg_data();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(post_chat_message).configure(configure_chat_message(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/chat_messages")
            .insert_header(header_auth(&token))
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST); // 400

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("text/plain; charset=utf-8"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let body_str = String::from_utf8_lossy(&body);
        assert!(body_str.contains(MSG_CONTENT_TYPE_ERROR));
    }
    #[actix_web::test]
    async fn test_post_chat_message_empty_json() {
        let (cfg_c, data_c, token) = get_cfg_data();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(post_chat_message).configure(configure_chat_message(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/chat_messages")
            .insert_header(header_auth(&token))
            .set_json(json!({}))
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST); // 400

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("text/plain; charset=utf-8"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let body_str = String::from_utf8_lossy(&body);
        let expected_message = "Json deserialize error: missing field";
        assert!(body_str.contains(expected_message));
    }
    #[actix_web::test]
    async fn test_post_chat_message_msg_min() {
        let (cfg_c, data_c, token) = get_cfg_data();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(post_chat_message).configure(configure_chat_message(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/chat_messages")
            .insert_header(header_auth(&token))
            .set_json(CreateChatMessageDto { stream_id: 1, msg: ChatMessageTest:: message_min() })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<AppError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        check_app_err(app_err_vec, err::CD_VALIDATION, &[chat_message_models::MSG_MESSAGE_MIN_LENGTH]);
    }
    #[actix_web::test]
    async fn test_post_chat_message_msg_max() {
        let (cfg_c, data_c, token) = get_cfg_data();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(post_chat_message).configure(configure_chat_message(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::post().uri("/api/chat_messages")
            .insert_header(header_auth(&token))
            .set_json(CreateChatMessageDto { stream_id: 1, msg: ChatMessageTest:: message_max() })
            .to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::EXPECTATION_FAILED); // 417

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let app_err_vec: Vec<AppError> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);
        #[rustfmt::skip]
        check_app_err(app_err_vec, err::CD_VALIDATION, &[chat_message_models::MSG_MESSAGE_MAX_LENGTH]);
    }

    // PUT /api/chat_messages    //.service(put_chat_message);
}
