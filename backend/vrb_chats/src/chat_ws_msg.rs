use actix::prelude::*;
use actix_broker::BrokerIssue;
use actix_web::http::StatusCode;
use actix_web_actors::ws;
use log::debug;
use serde_json::to_string;
use vrb_common::{
    api_error::{ApiError, code_to_str},
    err,
};

use crate::{
    chat_event_ws::{EWSType, ErrEWS, EventWS, MsgEWS},
    chat_message::SendMessage,
    chat_message_models::ChatMessage,
    chat_ws_assistant::AssistantChatMsg,
    chat_ws_async_result::AsyncResultError,
    chat_ws_session::ChatWsSession,
    chat_ws_tools,
};

#[derive(Debug, Clone)]
pub struct ChatWsMsgInfo {
    room_id: i32,
    user_id: i32,
    user_name: String,
    is_blocked: bool,
}

impl ChatWsMsgInfo {
    #[rustfmt::skip]
    pub fn new(room_id: i32, user_id: i32, user_name: String, is_blocked: bool) -> ChatWsMsgInfo {
        ChatWsMsgInfo { room_id, user_id, user_name, is_blocked }
    }
}

// Functionality for processing "chat message transfer" commands.

pub trait ChatWsMsg {
    fn get_msg_info(&self) -> ChatWsMsgInfo;

    fn handle_event_ews_msg(
        &self,
        event: EventWS,
        fn_chat_msg: impl AssistantChatMsg + 'static,
        ctx: &mut ws::WebsocketContext<ChatWsSession>,
    ) -> Result<bool, ErrEWS>
    where
        ChatWsSession: actix::Actor<Context = ws::WebsocketContext<ChatWsSession>>,
    {
        match event.ews_type() {
            EWSType::Msg => {
                // {"msg":"text msg"}
                let msg = event.get_string("msg").unwrap_or_default();
                self.handle_ews_msg_add_task(&msg, fn_chat_msg, ctx)?;
                Ok(true)
            }
            EWSType::MsgCut => {
                // {"msgCut": "", "id": 1}
                let id = event.get_i32("id").unwrap_or_default(); // (0);
                self.handle_ews_msg_cut_add_task(id, fn_chat_msg, ctx)?;
                Ok(true)
            }
            EWSType::MsgPut => {
                // {"msgPut": "modify msg", "id": 1}
                let msg_put = event.get_string("msgPut").unwrap_or_default();
                let id = event.get_i32("id").unwrap_or_default();
                self.handle_ews_msg_put_add_task(&msg_put, id, fn_chat_msg, ctx)?;
                Ok(true)
            }
            EWSType::MsgRmv => {
                // {"msgRmv": 1}
                let msg_rmv = event.get_i32("msgRmv").unwrap_or_default();
                self.handle_ews_msg_rmv_add_task(msg_rmv, fn_chat_msg, ctx)?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    // * Send a text message to all clients in the room. (Server -> Session) *
    fn handle_ews_msg_add_task(
        &self,
        msg: &str,
        fn_chat_msg: impl AssistantChatMsg + 'static,
        ctx: &mut ws::WebsocketContext<ChatWsSession>,
    ) -> Result<(), ErrEWS>
    where
        ChatWsSession: actix::Actor<Context = ws::WebsocketContext<ChatWsSession>>,
    {
        let msg_info = self.get_msg_info();
        let room_id = msg_info.room_id;
        let user_name = msg_info.user_name.clone();
        let msg = msg.to_owned();
        debug!("handle_ews_msg_add_task() room_id: {room_id}, user_name: {user_name}, msg: {msg}");
        // Check if this field is not empty
        chat_ws_tools::check_is_not_empty(&msg, "msg")?;
        // Check if there is an joined room
        chat_ws_tools::check_is_joined_room(room_id)?;
        // Check if there is a block on sending messages
        chat_ws_tools::check_is_blocked(msg_info.is_blocked)?;

        let user_id = msg_info.user_id;
        // Get room (stream) ID and user ID.
        let stream_id = room_id;
        // Spawn an async task.
        let addr = ctx.address();
        // Start an additional asynchronous task.
        actix_web::rt::spawn(async move {
            // Create a new user message in the chat.
            let result = execute_create_chat_message(stream_id, user_id, &msg, fn_chat_msg).await;
            if let Err(err) = result {
                return addr.do_send(AsyncResultError(err.status, err.code.to_string(), err.message.to_string()));
            }
            let opt_chat_message = result.unwrap();
            if opt_chat_message.is_none() {
                let message = format!("{}; stream_id: {}", err::MSG_STREAM_NOT_FOUND, stream_id);
                return addr.do_send(AsyncResultError(404, code_to_str(StatusCode::NOT_FOUND), message.to_string()));
            }
            let ch_msg = opt_chat_message.unwrap();
            // Send the "AsyncResultSendText" command for execution.
            addr.do_send(AsyncResultSendText(room_id, to_string(&MsgEWS::from(ch_msg)).unwrap()));
        });
        Ok(())
    }

    // * Send a delete message to all chat members. (Server -> Session) *
    fn handle_ews_msg_cut_add_task(
        &self,
        id: i32,
        fn_chat_msg: impl AssistantChatMsg + 'static,
        ctx: &mut ws::WebsocketContext<ChatWsSession>,
    ) -> Result<(), ErrEWS>
    where
        ChatWsSession: actix::Actor<Context = ws::WebsocketContext<ChatWsSession>>,
    {
        let msg_info = self.get_msg_info();
        let room_id = msg_info.room_id;
        let user_name = msg_info.user_name.clone();
        debug!("handle_ews_msg_cut_add_task() room_id: {room_id}, user_name: {user_name}, msg_id: {id}");
        let msg_cut = "".to_owned();
        // Check if this field is required
        chat_ws_tools::check_is_greater_than(id, 0, "id")?;
        // Check if there is an joined room
        chat_ws_tools::check_is_joined_room(room_id)?;
        // Check if there is a block on sending messages
        chat_ws_tools::check_is_blocked(msg_info.is_blocked)?;

        let user_id = msg_info.user_id;
        // Spawn an async task.
        let addr = ctx.address();
        // Start an additional asynchronous task.
        actix_web::rt::spawn(async move {
            // Change a user's message in a chat.
            let result = execute_modify_chat_message(id, user_id, &msg_cut, fn_chat_msg).await;
            if let Err(err) = result {
                return addr.do_send(AsyncResultError(err.status, err.code.to_string(), err.message.to_string()));
            }
            let opt_chat_message = result.unwrap();
            if opt_chat_message.is_none() {
                let message = format!("{}; id: {}, user_id: {}", err::MSG_CHAT_MESSAGE_NOT_FOUND, id, user_id);
                return addr.do_send(AsyncResultError(404, code_to_str(StatusCode::NOT_FOUND), message.to_string()));
            }
            let ch_msg = opt_chat_message.unwrap();
            // Send the "AsyncResultSendText" command for execution.
            addr.do_send(AsyncResultSendText(room_id, to_string(&MsgEWS::from(ch_msg)).unwrap()));
        });
        Ok(())
    }

    // * Send a correction to the message to everyone in the chat. (Server -> Session) *
    fn handle_ews_msg_put_add_task(
        &self,
        msg_put: &str,
        id: i32,
        fn_chat_msg: impl AssistantChatMsg + 'static,
        ctx: &mut ws::WebsocketContext<ChatWsSession>,
    ) -> Result<(), ErrEWS>
    where
        ChatWsSession: actix::Actor<Context = ws::WebsocketContext<ChatWsSession>>,
    {
        let msg_info = self.get_msg_info();
        let room_id = msg_info.room_id;
        let user_name = msg_info.user_name.clone();
        debug!("handle_ews_msg_put_add_task() room_id: {room_id}, user_name: {user_name}, msg_id: {id}, msg_put: {msg_put}");
        let msg_put = msg_put.to_owned();
        // Check if this field is not empty
        chat_ws_tools::check_is_not_empty(&msg_put, "msgPut")?;
        // Check if this field is required
        chat_ws_tools::check_is_greater_than(id, 0, "id")?;
        // Check if there is an joined room
        chat_ws_tools::check_is_joined_room(room_id)?;
        // Check if there is a block on sending messages
        chat_ws_tools::check_is_blocked(msg_info.is_blocked)?;

        let user_id = msg_info.user_id;
        // Spawn an async task.
        let addr = ctx.address();
        // Start an additional asynchronous task.
        actix_web::rt::spawn(async move {
            // Change a user's message in a chat.
            let result = execute_modify_chat_message(id, user_id, &msg_put, fn_chat_msg).await;
            if let Err(err) = result {
                return addr.do_send(AsyncResultError(err.status, err.code.to_string(), err.message.to_string()));
            }
            let opt_chat_message = result.unwrap();
            if opt_chat_message.is_none() {
                let message = format!("{}; id: {}, user_id: {}", err::MSG_CHAT_MESSAGE_NOT_FOUND, id, user_id);
                return addr.do_send(AsyncResultError(404, code_to_str(StatusCode::NOT_FOUND), message.to_string()));
            }
            let ch_msg = opt_chat_message.unwrap();
            // Send the "AsyncResultSendText" command for execution.
            addr.do_send(AsyncResultSendText(room_id, to_string(&MsgEWS::from(ch_msg)).unwrap()));
        });
        Ok(())
    }

    // * Send a permanent deletion message to all chat members. *
    fn handle_ews_msg_rmv_add_task(
        &self,
        msg_rmv: i32,
        fn_chat_msg: impl AssistantChatMsg + 'static,
        ctx: &mut ws::WebsocketContext<ChatWsSession>,
    ) -> Result<(), ErrEWS>
    where
        ChatWsSession: actix::Actor<Context = ws::WebsocketContext<ChatWsSession>>,
    {
        let msg_info = self.get_msg_info();
        let room_id = msg_info.room_id;
        let user_name = msg_info.user_name.clone();
        debug!("handle_ews_msg_rmv_add_task() room_id: {room_id}, user_name: {user_name}, msg_rmv: {msg_rmv}");
        // Check if this field is required
        chat_ws_tools::check_is_greater_than(msg_rmv, 0, "msgRmv")?;
        // Check if there is an joined room
        chat_ws_tools::check_is_joined_room(room_id)?;
        // Check if there is a block on sending messages
        chat_ws_tools::check_is_blocked(msg_info.is_blocked)?;

        let user_id = msg_info.user_id;
        // Spawn an async task.
        let addr = ctx.address();
        // Start an additional asynchronous task.
        actix_web::rt::spawn(async move {
            // Delete a user's message in a chat.
            let result = execute_delete_chat_message(msg_rmv, user_id, fn_chat_msg).await;
            if let Err(err) = result {
                return addr.do_send(AsyncResultError(err.status, err.code.to_string(), err.message.to_string()));
            }
            let opt_chat_message = result.unwrap();
            if opt_chat_message.is_none() {
                let message = format!("{}; id: {}, user_id: {}", err::MSG_CHAT_MESSAGE_NOT_FOUND, msg_rmv, user_id);
                return addr.do_send(AsyncResultError(404, code_to_str(StatusCode::NOT_FOUND), message.to_string()));
            }
            let ch_msg = opt_chat_message.unwrap();
            // Send the "AsyncResultSendText" command for execution.
            addr.do_send(AsyncResultSendText(room_id, to_string(&MsgEWS::from(ch_msg)).unwrap()));
        });
        Ok(())
    }
}

async fn execute_create_chat_message(
    stream_id: i32,
    user_id: i32,
    msg: &str,
    fn_chat_msg: impl AssistantChatMsg + 'static,
) -> Result<Option<ChatMessage>, ApiError> {
    fn_chat_msg.execute_create_chat_message(stream_id, user_id, &msg)
}

async fn execute_modify_chat_message(
    id: i32,
    user_id: i32,
    new_msg: &str,
    fn_chat_msg: impl AssistantChatMsg + 'static,
) -> Result<Option<ChatMessage>, ApiError> {
    fn_chat_msg.execute_modify_chat_message(id, user_id, new_msg)
}

async fn execute_delete_chat_message(
    id: i32,
    user_id: i32,
    fn_chat_msg: impl AssistantChatMsg + 'static,
) -> Result<Option<ChatMessage>, ApiError> {
    fn_chat_msg.execute_delete_chat_message(id, user_id)
}

// * * * * Handler for asynchronous response to the "SendText" event * * * *

struct AsyncResultSendText(
    i32,    // room_id
    String, // message text
);

impl Message for AsyncResultSendText {
    type Result = ();
}

impl Handler<AsyncResultSendText> for ChatWsSession {
    type Result = ();

    fn handle(&mut self, info: AsyncResultSendText, _ctx: &mut Self::Context) {
        // issue_async comes from having the `BrokerIssue` trait in scope.
        self.issue_system_async(SendMessage(info.0, info.1));
    }
}
