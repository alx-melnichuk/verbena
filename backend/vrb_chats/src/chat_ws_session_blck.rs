use actix::Addr;
use actix_web::http::StatusCode;
use log::debug;
use vrb_common::{
    api_error::{ApiError, code_to_str},
    err,
};

use crate::{
    chat_event_ws::{EWSType, ErrEWS, EventWS},
    chat_message_models::BlockedUser,
    chat_ws_assistant::AssistantBlockUser,
    chat_ws_async_result::{AsyncResultBlockClient, AsyncResultError},
    chat_ws_session::ChatWsSession,
    chat_ws_tools,
};

#[derive(Debug, Clone)]
pub struct ChatWsSessionBlckInfo {
    room_id: i32,
    user_id: i32,
    is_owner: bool,
}

impl ChatWsSessionBlckInfo {
    pub fn new(room_id: i32, user_id: i32, is_owner: bool) -> ChatWsSessionBlckInfo {
        ChatWsSessionBlckInfo {
            room_id,
            user_id,
            is_owner,
        }
    }
}

pub trait ChatWsSessionBlck {
    fn get_blck_info(&self) -> ChatWsSessionBlckInfo;

    fn handle_event_ews_blck(
        &self,
        event: EventWS,
        addr: Addr<ChatWsSession>,
        fn_block_user: impl AssistantBlockUser + 'static,
    ) -> Result<(), ErrEWS> {
        match event.ews_type() {
            EWSType::Block => {
                // {"block": "User2"}
                let block = event.get_string("block").unwrap_or("".to_owned());
                self.handle_ews_block_add_task(&block, true, addr, fn_block_user)?;
                Ok(())
            }
            EWSType::Unblock => {
                // {"unblock": "User2"}
                let block = event.get_string("unblock").unwrap_or("".to_owned());
                self.handle_ews_block_add_task(&block, false, addr, fn_block_user)?;
                Ok(())
            }
            _ => Ok(()),
        }
    }

    // ** Blocking clients in a room by name. (Session -> Server) **
    fn handle_ews_block_add_task(
        &self,
        user_name: &str,
        is_block: bool,
        addr: Addr<ChatWsSession>,
        fn_block_user: impl AssistantBlockUser + 'static,
    ) -> Result<(), ErrEWS> {
        let blck_info = self.get_blck_info();
        let room_id = blck_info.room_id;
        debug!("handle_ews_block_add_task() room_id: {room_id}, user_name: {user_name}, is_block: {is_block}");
        let tag_name = if is_block { "block" } else { "unblock" };
        // Check if this field is not empty
        chat_ws_tools::check_is_not_empty(user_name, tag_name)?;
        // Check if there is an joined room
        chat_ws_tools::check_is_joined_room(room_id)?;
        // Check if the user is the owner of the stream.
        chat_ws_tools::check_is_owner_room(blck_info.is_owner)?;

        let user_id = blck_info.user_id;
        let block_name = user_name.to_string();
        // Start an additional asynchronous task.
        actix_web::rt::spawn(async move {
            let blocked_nickname = block_name.clone();
            // Perform blocking/unblocking of a user.
            let result = execute_block_user(is_block, user_id, None, Some(block_name), fn_block_user).await;
            if let Err(err) = result {
                return addr.do_send(AsyncResultError(err.status, err.code.to_string(), err.message.to_string()));
            }
            let opt_blocked_user = result.unwrap();
            if opt_blocked_user.is_none() {
                let message = format!("{}; blocked_nickname: '{}'", err::MSG_USER_NOT_FOUND, &blocked_nickname);
                return addr.do_send(AsyncResultError(404, code_to_str(StatusCode::NOT_FOUND), message.to_string()));
            }
            let blocked_user = opt_blocked_user.unwrap();
            let blocked_name = blocked_user.blocked_nickname.clone();
            addr.do_send(AsyncResultBlockClient(room_id, is_block, blocked_name));
        });
        Ok(())
    }
}

async fn execute_block_user(
    is_block: bool,
    user_id: i32,
    blocked_id: Option<i32>,
    blocked_nickname: Option<String>,
    fn_block_user: impl AssistantBlockUser + 'static,
) -> Result<Option<BlockedUser>, ApiError> {
    fn_block_user.execute_block_user(is_block, user_id, blocked_id, blocked_nickname)
}
