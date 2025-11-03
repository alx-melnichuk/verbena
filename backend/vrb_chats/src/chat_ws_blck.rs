use actix::prelude::*;
use actix_web::http::StatusCode;
use actix_web_actors::ws;
use log::debug;
use serde_json::to_string;
use vrb_common::{
    api_error::{ApiError, code_to_str},
    err,
};

use crate::{
    chat_event_ws::{BlockEWS, EWSType, ErrEWS, EventWS, UnblockEWS},
    chat_message::{BlockClient, BlockSsn},
    chat_message_models::BlockedUserMini,
    chat_ws_assistant::AssistantBlockUser,
    chat_ws_async_result::{AsyncResultBlockClient, AsyncResultError},
    chat_ws_server::ChatWsServer,
    chat_ws_session::ChatWsSession,
    chat_ws_tools,
};

#[derive(Debug, Clone)]
pub struct ChatWsBlckInfo {
    room_id: i32,
    user_id: i32,
    user_name: String,
    is_owner: bool,
}

impl ChatWsBlckInfo {
    #[rustfmt::skip]
    pub fn new(room_id: i32, user_id: i32, user_name: String, is_owner: bool) -> ChatWsBlckInfo {
        ChatWsBlckInfo { room_id, user_id, user_name, is_owner }
    }
}

// ** Functionality for handling commands to block/unblock chat members. **

pub trait ChatWsBlck {
    fn get_blck_info(&self) -> ChatWsBlckInfo;

    fn set_is_blocked(&mut self, is_blocked: bool);

    fn handle_event_ews_blck(
        &self,
        event: EventWS,
        fn_block_user: impl AssistantBlockUser + 'static,
        ctx: &mut ws::WebsocketContext<ChatWsSession>,
    ) -> Result<bool, ErrEWS>
    where
        ChatWsSession: actix::Actor<Context = ws::WebsocketContext<ChatWsSession>>,
    {
        match event.ews_type() {
            EWSType::Block => {
                // {"block": "User2"}
                let block = event.get_string("block").unwrap_or("".to_owned());
                self.handle_ews_block_add_task(&block, true, fn_block_user, ctx)?;
                Ok(true)
            }
            EWSType::Unblock => {
                // {"unblock": "User2"}
                let block = event.get_string("unblock").unwrap_or("".to_owned());
                self.handle_ews_block_add_task(&block, false, fn_block_user, ctx)?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    // ** Blocking clients in a room by name. (Session -> Server) **
    fn handle_ews_block_add_task(
        &self,
        user_name: &str,
        is_block: bool,
        fn_block_user: impl AssistantBlockUser + 'static,
        ctx: &mut ws::WebsocketContext<ChatWsSession>,
    ) -> Result<(), ErrEWS>
    where
        ChatWsSession: actix::Actor<Context = ws::WebsocketContext<ChatWsSession>>,
    {
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
        let addr = ctx.address();
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
            let blocked_name = blocked_user.nickname.clone();
            addr.do_send(AsyncResultBlockClient(room_id, is_block, blocked_name));
        });
        Ok(())
    }

    // Handler for "CommandSrv::Block(BlockSsn)".
    fn handle_commandsrv_block(&mut self, block: BlockSsn, ctx: &mut ws::WebsocketContext<ChatWsSession>) {
        let BlockSsn(is_block, is_in_chat) = block;
        self.set_is_blocked(is_block);
        let blck_info = self.get_blck_info();
        let user_name = blck_info.user_name.clone();
        #[rustfmt::skip]
        let str = if is_block {
            to_string(&BlockEWS { block: user_name, is_in_chat }).unwrap()
        } else {
            to_string(&UnblockEWS { unblock: user_name, is_in_chat }).unwrap()
        };
        debug!("handler<CommandSrv::Block>() is_block: {is_block}, str: {str}");
        ctx.text(str);
    }
}

async fn execute_block_user(
    is_block: bool,
    user_id: i32,
    blocked_id: Option<i32>,
    blocked_nickname: Option<String>,
    fn_block_user: impl AssistantBlockUser + 'static,
) -> Result<Option<BlockedUserMini>, ApiError> {
    fn_block_user.execute_block_user(is_block, user_id, blocked_id, blocked_nickname)
}

// * * * * Handler for asynchronous response to the "BlockClient" event * * * *

impl Message for AsyncResultBlockClient {
    type Result = ();
}

impl Handler<AsyncResultBlockClient> for ChatWsSession {
    type Result = ();

    fn handle(&mut self, info: AsyncResultBlockClient, ctx: &mut Self::Context) {
        let AsyncResultBlockClient(room_id, is_block, blocked_name) = info;
        let block_client = BlockClient(room_id, blocked_name.clone(), is_block);

        ChatWsServer::from_registry()
            .send(block_client)
            .into_actor(self)
            .then(move |res, _act, ctx| {
                if let Ok(is_in_chat) = res {
                    #[rustfmt::skip]
                    let str = if is_block {
                        to_string(&BlockEWS { block: blocked_name, is_in_chat }).unwrap()
                    } else {
                        to_string(&UnblockEWS { unblock: blocked_name, is_in_chat }).unwrap()
                    };
                    debug!("handler<AsyncResultBlockClient>() is_block: {is_block}, str: {str}");
                    ctx.text(str);
                }
                fut::ready(())
            })
            .wait(ctx);
    }
}

// * * * *  __  * * * *
