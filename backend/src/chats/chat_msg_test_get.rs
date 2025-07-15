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
    use chrono::SecondsFormat;
    use serde_json;

    use crate::chats::{
        chat_message_controller::{
            get_chat_message,
            tests::{configure_chat_message, get_cfg_data, header_auth, MSG_FAILED_DESER},
        },
        chat_message_models::ChatMessageDto,
        chat_message_orm::tests::ChatMsgTest,
    };

    // ** get_chat_message **

    #[actix_web::test]
    async fn test_get_chat_message_search_by_str_id() {
        let (cfg_c, data_c, token) = get_cfg_data(4);
        let stream_id = ChatMsgTest::stream_ids().get(0).unwrap().clone();
        #[rustfmt::skip]
        let ch_msg1_dto_vec: Vec<ChatMessageDto> = data_c.2.clone()
            .iter().map(|ch_msg| ChatMessageDto::from(ch_msg.clone())).collect();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(get_chat_message).configure(configure_chat_message(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get()
            .uri(&format!("/api/chat_messages?streamId={}", stream_id))
            .insert_header(header_auth(&token)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK); // 200

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let response: Vec<ChatMessageDto> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        let json = serde_json::json!(&ch_msg1_dto_vec).to_string();
        let ch_msg1_vec_ser: Vec<ChatMessageDto> = serde_json::from_slice(json.as_bytes()).expect(MSG_FAILED_DESER);
        assert_eq!(response.len(), ch_msg1_vec_ser.len());
        assert_eq!(response, ch_msg1_vec_ser);
    }
    #[actix_web::test]
    async fn test_get_chat_message_search_by_str_id_sort_des() {
        let (cfg_c, data_c, token) = get_cfg_data(4);
        let stream_id = ChatMsgTest::stream_ids().get(0).unwrap().clone();
        #[rustfmt::skip]
        let ch_msg1_dto_vec: Vec<ChatMessageDto> = data_c.2.clone()
            .iter().map(|ch_msg| ChatMessageDto::from(ch_msg.clone())).rev().collect();
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(get_chat_message).configure(configure_chat_message(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get()
            .uri(&format!("/api/chat_messages?streamId={}&isSortDes=true", stream_id))
            .insert_header(header_auth(&token)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK); // 200

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let response: Vec<ChatMessageDto> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        let json = serde_json::json!(&ch_msg1_dto_vec).to_string();
        let ch_msg1_vec_ser: Vec<ChatMessageDto> = serde_json::from_slice(json.as_bytes()).expect(MSG_FAILED_DESER);
        assert_eq!(response.len(), ch_msg1_vec_ser.len());
        assert_eq!(response, ch_msg1_vec_ser);
    }
    #[actix_web::test]
    async fn test_get_chat_message_search_by_str_id_part1() {
        let (cfg_c, data_c, token) = get_cfg_data(4);
        let stream_id = ChatMsgTest::stream_ids().get(0).unwrap().clone();
        let limit = 3;
        #[rustfmt::skip]
        let ch_msg_dto_vec: Vec<ChatMessageDto> = data_c.2.clone()
            .iter().map(|ch_msg| ChatMessageDto::from(ch_msg.clone())).collect();

        let ch_msg1_dto_vec = &ch_msg_dto_vec[0..limit];
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(get_chat_message).configure(configure_chat_message(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get()
            .uri(&format!("/api/chat_messages?streamId={}&limit={}", stream_id, limit))
            .insert_header(header_auth(&token)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK); // 200

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let response: Vec<ChatMessageDto> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        let json = serde_json::json!(&ch_msg1_dto_vec).to_string();
        let ch_msg1_vec_ser: Vec<ChatMessageDto> = serde_json::from_slice(json.as_bytes()).expect(MSG_FAILED_DESER);
        assert_eq!(response.len(), ch_msg1_vec_ser.len());
        assert_eq!(response, ch_msg1_vec_ser);
    }
    #[actix_web::test]
    async fn test_get_chat_message_search_by_str_id_part2() {
        let (cfg_c, data_c, token) = get_cfg_data(4);
        let stream_id = ChatMsgTest::stream_ids().get(0).unwrap().clone();
        let limit = 3;
        #[rustfmt::skip]
        let ch_msg_dto_vec: Vec<ChatMessageDto> = data_c.2.clone()
            .iter().map(|ch_msg| ChatMessageDto::from(ch_msg.clone())).collect();

        let min_date = ch_msg_dto_vec.get(limit - 1).unwrap().date;
        let min_date_str = min_date.to_rfc3339_opts(SecondsFormat::Millis, true);
        let ch_msg1_dto_vec = &ch_msg_dto_vec[limit..2 * limit];
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(get_chat_message).configure(configure_chat_message(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get()
            .uri(&format!("/api/chat_messages?streamId={}&minDate={}&limit={}", stream_id, min_date_str, limit))
            .insert_header(header_auth(&token)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK); // 200

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let response: Vec<ChatMessageDto> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        let json = serde_json::json!(&ch_msg1_dto_vec).to_string();
        let ch_msg1_vec_ser: Vec<ChatMessageDto> = serde_json::from_slice(json.as_bytes()).expect(MSG_FAILED_DESER);
        assert_eq!(response.len(), ch_msg1_vec_ser.len());
        assert_eq!(response, ch_msg1_vec_ser);
    }
    #[actix_web::test]
    async fn test_get_chat_message_search_by_str_id_sort_des_part1() {
        let (cfg_c, data_c, token) = get_cfg_data(4);
        let stream_id = ChatMsgTest::stream_ids().get(0).unwrap().clone();
        let limit = 3;
        #[rustfmt::skip]
        let ch_msg_dto_vec: Vec<ChatMessageDto> = data_c.2.clone()
            .iter().map(|ch_msg| ChatMessageDto::from(ch_msg.clone())).rev().collect();

        let ch_msg1_dto_vec = &ch_msg_dto_vec[0..limit];
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(get_chat_message).configure(configure_chat_message(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get()
            .uri(&format!("/api/chat_messages?streamId={}&isSortDes=true&limit={}", stream_id, limit))
            .insert_header(header_auth(&token)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK); // 200

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let response: Vec<ChatMessageDto> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        let json = serde_json::json!(&ch_msg1_dto_vec).to_string();
        let ch_msg1_vec_ser: Vec<ChatMessageDto> = serde_json::from_slice(json.as_bytes()).expect(MSG_FAILED_DESER);
        assert_eq!(response.len(), ch_msg1_vec_ser.len());
        assert_eq!(response, ch_msg1_vec_ser);
    }
    #[actix_web::test]
    async fn test_get_chat_message_search_by_str_id_sort_des_part2() {
        let (cfg_c, data_c, token) = get_cfg_data(4);
        let stream_id = ChatMsgTest::stream_ids().get(0).unwrap().clone();
        let limit = 3;
        #[rustfmt::skip]
        let ch_msg_dto_vec: Vec<ChatMessageDto> = data_c.2.clone()
            .iter().map(|ch_msg| ChatMessageDto::from(ch_msg.clone())).rev().collect();

        let max_date = ch_msg_dto_vec.get(limit - 1).unwrap().date;
        let max_date_str = max_date.to_rfc3339_opts(SecondsFormat::Millis, true);
        let ch_msg1_dto_vec = &ch_msg_dto_vec[limit..2 * limit];
        #[rustfmt::skip]
        let app = test::init_service(
            App::new().service(get_chat_message).configure(configure_chat_message(cfg_c, data_c))).await;
        #[rustfmt::skip]
        let req = test::TestRequest::get()
            .uri(&format!("/api/chat_messages?streamId={}&isSortDes=true&maxDate={}&limit={}", stream_id, max_date_str, limit))
            .insert_header(header_auth(&token)).to_request();
        let resp: dev::ServiceResponse = test::call_service(&app, req).await;
        assert_eq!(resp.status(), StatusCode::OK); // 200

        #[rustfmt::skip]
        assert_eq!(resp.headers().get(CONTENT_TYPE).unwrap(), HeaderValue::from_static("application/json"));
        let body = body::to_bytes(resp.into_body()).await.unwrap();
        let response: Vec<ChatMessageDto> = serde_json::from_slice(&body).expect(MSG_FAILED_DESER);

        let json = serde_json::json!(&ch_msg1_dto_vec).to_string();
        let ch_msg1_vec_ser: Vec<ChatMessageDto> = serde_json::from_slice(json.as_bytes()).expect(MSG_FAILED_DESER);
        assert_eq!(response.len(), ch_msg1_vec_ser.len());
        assert_eq!(response, ch_msg1_vec_ser);
    }
}
