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
    // use chrono::{SecondsFormat, Utc};
    use serde_json;

    use crate::chats::{
        chat_message_controller::{
            get_chat_message,
            tests::{
                configure_chat_message, get_cfg_data,
                header_auth,      /*MSG_CASTING_TO_TYPE, MSG_CONTENT_TYPE_ERROR,*/
                MSG_FAILED_DESER, /*MSG_JSON_MISSING_FIELD,*/
            },
        },
        chat_message_models::ChatMessageDto,
        chat_message_orm::tests::ChatMsgTest,
    };
    // use crate::errors::AppError;
    // use crate::settings::err;
    // use crate::users::user_models::UserRole;

    // ** get_chat_message **

    #[actix_web::test]
    async fn test_get_chat_message_valid_data() {
        let (cfg_c, data_c, token) = get_cfg_data(4);
        // let user1_name = data_c.0.get(0).unwrap().nickname.clone();
        let ch_msg_vec = data_c.2.clone();
        // let ch_msg1_vec = &ch_msg_vec[0..2];
        let ch_msg1_dto_vec: Vec<ChatMessageDto> =
            ch_msg_vec.iter().map(|ch_msg| ChatMessageDto::from(ch_msg.clone())).collect();

        let stream_id = ChatMsgTest::stream_ids().get(0).unwrap().clone();
        // let limit = 3;
        // eprintln!("");
        // eprintln!("data_c.2: {:#?}", ch_msg_vec);
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(get_chat_message).configure(configure_chat_message(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get()
            // .uri(&format!("/api/chat_messages?streamId={}&limit={}", stream_id, limit))
            .uri(&format!("/api/chat_messages?streamId={}", stream_id))
            .insert_header(header_auth(&token)).to_request();

        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK); // 200

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        // eprintln!("body: {:#?}", body);
        let response: Vec<ChatMessageDto> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        let json = serde_json::json!(&ch_msg1_dto_vec).to_string();
        let ch_msg1_vec_ser: Vec<ChatMessageDto> = serde_json::from_slice(json.as_bytes()).expect(MSG_FAILED_DESER);

        assert_eq!(response.len(), ch_msg1_vec_ser.len());
        assert_eq!(response, ch_msg1_vec_ser);
    }
}
