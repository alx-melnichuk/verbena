use log::{Level::Debug, debug, log_enabled};

use actix::prelude::*;
use actix_broker::{BrokerIssue, BrokerMsg};
use actix_web::http::StatusCode;
use actix_web_actors::ws::{self, CloseReason};
use serde_json::to_string;
use vrb_common::{api_error::code_to_str, err};

use crate::{
    chat_event_ws::{BlockEWS, CountEWS, EWSType, EchoEWS, ErrEWS, EventWS, JoinEWS, MsgEWS, MsgRmvEWS, NameEWS, UnblockEWS},
    chat_message::{BlockClient, BlockSsn, ChatMsgSsn, CommandSrv, CountMembers, JoinRoom, LeaveRoom, SendMessage},
    chat_ws_assistant::ChatWsAssistant,
    chat_ws_server::ChatWsServer,
    chat_ws_session_prm::ChatWsSessionPrm,
    chat_ws_tools,
};

pub const SPECIFIED_USER_NOT_FOUND: &str = "The specified user was not found.";

// ** ChatWsSession **

pub struct ChatWsSession {
    id: u64,
    room_id: i32,
    user_id: i32,
    user_name: String,
    is_owner: bool,
    is_blocked: bool,
    assistant: ChatWsAssistant,
}

// ** ChatWsSession implementation "Actor" **

impl Actor for ChatWsSession {
    type Context = ws::WebsocketContext<Self>;
    // Called when an actor gets polled the first time.
    fn started(&mut self, _ctx: &mut Self::Context) {
        if log_enabled!(Debug) {
            let user_str = format!("user_id: {}, user_name: \"{}\", id: {}", self.user_id, &self.user_name, self.id);
            debug!("ChatWsSession.started() room_id {}, {}", self.room_id, user_str);
        }
    }
    // Called after an actor is stopped.
    fn stopped(&mut self, _ctx: &mut Self::Context) {
        if log_enabled!(Debug) {
            let user_str = format!("user_id: {}, user_name: \"{}\", id: {}", self.user_id, &self.user_name, self.id);
            debug!("ChatWsSession.stopped() room_id {}, {}", self.room_id, user_str);
        }
    }
}

// ** ChatWsSession implementation "StreamHandler<Result<ws::Message, ws::ProtocolError>>" **

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for ChatWsSession {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        let msg = match msg {
            Err(_) => {
                ctx.stop();
                return;
            }
            Ok(msg) => msg,
        };
        debug!("StreamHandler<Message>: {msg:?}");
        match msg {
            ws::Message::Text(text) => {
                // Handle socket text messages.
                self.handle_text_messages(text.trim(), ctx);
            }
            ws::Message::Close(reason) => {
                // Send a message about leaving the room.
                let leave_room_srv = LeaveRoom(self.room_id, self.id, self.user_name.clone());
                debug!("StreamHandler<Message::Close> issue_system_sync(leave_room_srv, ctx)");
                // issue_sync comes from having the `BrokerIssue` trait in scope.
                self.issue_system_sync(leave_room_srv, ctx);
                
                debug!("StreamHandler<Message::Close> actix_web::rt::spawn();");
                let addr = ctx.address();
                // Start an additional asynchronous task.
                actix_web::rt::spawn(async move {
                    // Spawns a future on the current thread as a new task.
                    // This is required so that two concurrent events arrive in this actor's event queue.
                    // One thread processed the LeaveRoom command.
                    // Another thread closed the socket.
                    // If these events are executed in the same thread, closing the socket clears the event queue.
                    // As a result, the LeaveRoom command is not processed.
                    debug!("StreamHandler<Message::Close> addr.do_send(CommandSrv::CloseAndStop(reason))");
                    addr.do_send(CommandSrv::CloseAndStop(reason));
                });
            }
            _ => {}
        }
    }
}

// ** ChatWsSession implementation **

impl ChatWsSession {
    pub fn new(
        id: u64,
        room_id: i32,
        user_id: i32,
        user_name: String,
        is_owner: bool,
        is_blocked: bool,
        assistant: ChatWsAssistant,
    ) -> Self {
        ChatWsSession {
            id,
            room_id,
            user_id,
            user_name,
            is_owner,
            is_blocked,
            assistant,
        }
    }
    /** Handle socket text messages. */
    fn handle_text_messages(&mut self, msg: &str, ctx: &mut ws::WebsocketContext<Self>) {
        // Parse input data of ws event.
        let res_event = EventWS::parsing(msg);
        if let Err(err) = res_event {
            debug!("handle_text_messages() error: {:?} msg: \"{}\"", err, msg);
            ctx.text(to_string(&chat_ws_tools::get_err500(&err)).unwrap());
            return;
        }
        let event = res_event.unwrap();

        match event.ews_type() {
            EWSType::Echo => {
                // {"echo": "text1"}
                let echo = event.get_string("echo").unwrap_or("".to_owned());
                // Check if this field is required
                if let Err(err) = chat_ws_tools::check_is_not_empty(&echo, "echo") {
                    ctx.text(to_string(&err).unwrap());
                } else {
                    ctx.text(to_string(&EchoEWS { echo }).unwrap());
                }
            }
            EWSType::Block => {
                // {"block": "User2"}
                let block = event.get_string("block").unwrap_or("".to_owned());
                if let Err(err) = self.handle_ews_block_add_task(&block, true, ctx) {
                    ctx.text(to_string(&err).unwrap());
                }
            }
            EWSType::Count => {
                // {"count": -1}
                if let Err(err) = self.handle_ews_count(ctx) {
                    ctx.text(to_string(&err).unwrap());
                }
            }
            EWSType::Join => {
                // {"join": 1}
                let room_id = event.get_i32("join").unwrap_or_default(); // (0);
                let access = event.get_string("access").unwrap_or("".to_owned());
                if let Err(err) = self.handle_ews_join_add_task(room_id, &access, ctx) {
                    ctx.text(to_string(&err).unwrap());
                }
            }
            EWSType::Leave => {
                // {"leave": -1}
                if let Err(err) = self.handle_ews_leave(ctx) {
                    ctx.text(to_string(&err).unwrap());
                }
            }
            EWSType::Msg => {
                // {"msg":"text msg"}
                let msg = event.get_string("msg").unwrap_or_default();
                if let Err(err) = self.handle_ews_msg_add_task(&msg, ctx) {
                    ctx.text(to_string(&err).unwrap());
                }
            }
            EWSType::MsgCut => {
                // {"msgCut": "", "id": 1}
                let id = event.get_i32("id").unwrap_or_default(); // (0);
                if let Err(err) = self.handle_ews_msg_cut_add_task(id, ctx) {
                    ctx.text(to_string(&err).unwrap());
                }
            }
            EWSType::MsgPut => {
                // {"msgPut": "modify msg", "id": 1}
                let msg_put = event.get_string("msgPut").unwrap_or_default();
                let id = event.get_i32("id").unwrap_or_default();
                if let Err(err) = self.handle_ews_msg_put_add_task(&msg_put, id, ctx) {
                    ctx.text(to_string(&err).unwrap());
                }
            }
            EWSType::MsgRmv => {
                // {"msgRmv": 1}
                let msg_rmv = event.get_i32("msgRmv").unwrap_or_default();
                if let Err(err) = self.handle_ews_msg_rmv_add_task(msg_rmv, ctx) {
                    ctx.text(to_string(&err).unwrap());
                }
            }
            EWSType::Name => {
                // {"name": "User1"}
                let new_name = event.get_string("name").unwrap_or("".to_owned());
                if let Err(err) = self.handle_ews_name(&new_name, ctx) {
                    ctx.text(to_string(&err).unwrap());
                }
            }
            EWSType::PrmBool | EWSType::PrmInt | EWSType::PrmStr => {
                if let Err(err) = self.handle_event_ews_type(event) {
                    ctx.text(to_string(&err).unwrap());
                }
            }
            EWSType::Unblock => {
                // {"unblock": "User2"}
                let block = event.get_string("unblock").unwrap_or("".to_owned());
                if let Err(err) = self.handle_ews_block_add_task(&block, false, ctx) {
                    ctx.text(to_string(&err).unwrap());
                }
            }
            _ => {}
        }
    }

    // ** Blocking clients in a room by name. (Session -> Server) **
    pub fn handle_ews_block_add_task(&self, user_name: &str, is_block: bool, ctx: &mut ws::WebsocketContext<Self>) -> Result<(), ErrEWS> {
        let room_id = self.room_id;
        debug!("handle_ews_block_add_task() room_id: {room_id}, user_name: {user_name}, is_block: {is_block}");
        let tag_name = if is_block { "block" } else { "unblock" };
        // Check if this field is not empty
        chat_ws_tools::check_is_not_empty(user_name, tag_name)?;
        // Check if there is an joined room
        chat_ws_tools::check_is_joined_room(self.room_id)?;
        // Check if the user is the owner of the stream.
        chat_ws_tools::check_is_owner_room(self.is_owner)?;

        let room_id = self.room_id;
        let user_id = self.user_id;
        let block_name = user_name.to_string();
        // Spawn an async task.
        let addr = ctx.address();
        let assistant = self.assistant.clone();
        // Start an additional asynchronous task.
        actix_web::rt::spawn(async move {
            let blocked_nickname = block_name.clone();
            // Perform blocking/unblocking of a user.
            let result = assistant.execute_block_user(is_block, user_id, None, Some(block_name)).await;
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

    // ** Count of clients in the room. (Session -> Server) **
    pub fn handle_ews_count(&mut self, ctx: &mut ws::WebsocketContext<Self>) -> Result<(), ErrEWS> {
        // Check if there is an joined room
        chat_ws_tools::check_is_joined_room(self.room_id)?;
        let count_members = CountMembers(self.room_id);

        ChatWsServer::from_registry()
            .send(count_members)
            .into_actor(self)
            .then(|res, _act, ctx| {
                if let Ok(count) = res {
                    ctx.text(to_string(&CountEWS { count }).unwrap());
                }
                fut::ready(())
            })
            .wait(ctx);
        Ok(())
    }

    // * Join the client to the chat room. (Session -> Server) *
    pub fn handle_ews_join_add_task(&mut self, room_id: i32, access: &str, ctx: &mut ws::WebsocketContext<Self>) -> Result<(), ErrEWS> {
        debug!("handle_ews_join_add_task() room_id: {room_id}, access.len(): {}", access.len());
        // Check if this field is required
        chat_ws_tools::check_is_greater_than(room_id, 0, "join")?;
        // Check if there was a join to this room.
        if self.room_id == room_id {
            return Err(chat_ws_tools::get_err409(err::MSG_THERE_WAS_ALREADY_JOIN_TO_ROOM));
        }

        let mut num_token: i32 = i32::default();
        let mut opt_user_id: Option<i32> = None;
        if access.len() > 0 {
            // Decode the token. And unpack the two parameters from the token.
            #[rustfmt::skip]
            let (user_id, num_token2) =
                self.assistant.decode_and_verify_token(access).map_err(|err| chat_ws_tools::get_err401(&err))?;
            opt_user_id = Some(user_id);
            num_token = num_token2;
        }

        // Spawn an async task.
        let addr = ctx.address();
        let assistant = self.assistant.clone();

        let mut user_name = self.user_name.clone();
        // Start an additional asynchronous task.
        actix_web::rt::spawn(async move {
            if let Some(user_id) = opt_user_id {
                // Check the correctness of the numeric token and get the user data.
                let result = assistant.check_num_token_and_get_user(user_id, num_token).await;
                if let Err(err) = result {
                    return addr.do_send(AsyncResultError(err.status, err.code.to_string(), err.message.to_string()));
                }
                user_name = result.unwrap().nickname.clone();
            }

            // Get chat access information.
            let result = assistant.get_chat_access(room_id, opt_user_id).await;
            if let Err(err) = result {
                return addr.do_send(AsyncResultError(err.status, err.code.to_string(), err.message.to_string()));
            }

            let opt_chat_access = result.unwrap();
            // If the stream with id = room_id is not found, then an error occurs.
            if opt_chat_access.is_none() {
                let message = format!("{}; stream_id: {}", err::MSG_STREAM_NOT_FOUND, room_id);
                return addr.do_send(AsyncResultError(404, code_to_str(StatusCode::NOT_FOUND), message.to_string()));
            }
            let chat_access = opt_chat_access.unwrap();
            // Check stream activity. ("state" IN ('waiting', 'preparing', 'started', 'paused') != 'stopped')
            // if chat_access.stream_state == "stopped" {
            //     let message = err::MSG_STREAM_NOT_ACTIVE.to_string();
            //     return addr.do_send(AsyncResultError(409, code_to_str(StatusCode::CONFLICT), message));
            // }
            // Determine if a user is the owner of a chat.
            #[rustfmt::skip]
            let is_owner = if opt_user_id.is_some() { opt_user_id.unwrap() == chat_access.stream_owner } else { false };
            // Get the "block" value for the given user.
            let is_blocked = if opt_user_id.is_some() { chat_access.is_blocked } else { true };

            let user_id = opt_user_id.unwrap_or(i32::default());
            debug!("handle_ews_join_add_task() room_id:{room_id}, user_name:{user_name}, is_owner:{is_owner}, is_blocked:{is_blocked}");
            // Send the "AsyncResultEwsJoin" command for execution.
            addr.do_send(AsyncResultEwsJoin(room_id, user_id, user_name, is_owner, is_blocked));
        });
        Ok(())
    }

    // * Leave the client from the chat room. (Session -> Server) *
    pub fn handle_ews_leave(&mut self, ctx: &mut ws::WebsocketContext<Self>) -> Result<(), ErrEWS> {
        let is_connected = ctx.address().connected();
        #[rustfmt::skip]
        debug!("handle_ews_leave() room_id: {}, user_name: {}, is_connected: {}", self.room_id, self.user_name, is_connected);
        // Check if there is an joined room
        chat_ws_tools::check_is_joined_room(self.room_id)?;

        // Send a message about leaving the room.
        let leave_room_srv = LeaveRoom(self.room_id, self.id, self.user_name.clone());
        // issue_sync comes from having the `BrokerIssue` trait in scope.
        self.issue_system_sync(leave_room_srv, ctx);
        Ok(())
    }

    // * Send a text message to all clients in the room. (Server -> Session) *
    pub fn handle_ews_msg_add_task(&self, msg: &str, ctx: &mut ws::WebsocketContext<Self>) -> Result<(), ErrEWS> {
        let msg = msg.to_owned();
        // Check if this field is not empty
        chat_ws_tools::check_is_not_empty(&msg, "msg")?;
        // Check if there is an joined room
        chat_ws_tools::check_is_joined_room(self.room_id)?;
        // Check if there is a block on sending messages
        chat_ws_tools::check_is_blocked(self.is_blocked)?;

        // Get room (stream) ID and user ID.
        let stream_id = self.room_id;
        let user_id = self.user_id;
        // Spawn an async task.
        let addr = ctx.address();
        let assistant = self.assistant.clone();
        // Start an additional asynchronous task.
        actix_web::rt::spawn(async move {
            // Create a new user message in the chat.
            let result = assistant.execute_create_chat_message(stream_id, user_id, &msg).await;
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
            addr.do_send(AsyncResultSendText(to_string(&MsgEWS::from(ch_msg)).unwrap()));
        });
        Ok(())
    }

    // * Send a delete message to all chat members. (Server -> Session) *
    pub fn handle_ews_msg_cut_add_task(&self, id: i32, ctx: &mut ws::WebsocketContext<Self>) -> Result<(), ErrEWS> {
        let msg_cut = "".to_owned();
        // Check if this field is required
        chat_ws_tools::check_is_greater_than(id, 0, "id")?;
        // Check if there is an joined room
        chat_ws_tools::check_is_joined_room(self.room_id)?;
        // Check if there is a block on sending messages
        chat_ws_tools::check_is_blocked(self.is_blocked)?;

        let user_id = self.user_id;
        // Spawn an async task.
        let addr = ctx.address();
        let assistant = self.assistant.clone();
        // Start an additional asynchronous task.
        actix_web::rt::spawn(async move {
            // Change a user's message in a chat.
            let result = assistant.execute_modify_chat_message(id, user_id, &msg_cut).await;
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
            addr.do_send(AsyncResultSendText(to_string(&MsgEWS::from(ch_msg)).unwrap()));
        });
        Ok(())
    }
    // * Send a correction to the message to everyone in the chat. (Server -> Session) *
    #[rustfmt::skip]
    pub fn handle_ews_msg_put_add_task(
        &self, msg_put: &str, id: i32, ctx: &mut ws::WebsocketContext<Self>
    ) -> Result<(), ErrEWS> {
        let msg_put = msg_put.to_owned();
        // Check if this field is not empty
        chat_ws_tools::check_is_not_empty(&msg_put, "msgPut")?;
        // Check if this field is required
        chat_ws_tools::check_is_greater_than(id, 0, "id")?;
        // Check if there is an joined room
        chat_ws_tools::check_is_joined_room(self.room_id)?;
        // Check if there is a block on sending messages
        chat_ws_tools::check_is_blocked(self.is_blocked)?;

        let user_id = self.user_id;
        // Spawn an async task.
        let addr = ctx.address();
        let assistant = self.assistant.clone();
        // Start an additional asynchronous task.
        actix_web::rt::spawn(async move {
            // Change a user's message in a chat.
            let result = assistant.execute_modify_chat_message(id, user_id, &msg_put).await;
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
            addr.do_send(AsyncResultSendText(to_string(&MsgEWS::from(ch_msg)).unwrap()));
        });
        Ok(())
    }
    // * Send a permanent deletion message to all chat members. *
    pub fn handle_ews_msg_rmv_add_task(&self, msg_rmv: i32, ctx: &mut ws::WebsocketContext<Self>) -> Result<(), ErrEWS> {
        // Check if this field is required
        chat_ws_tools::check_is_greater_than(msg_rmv, 0, "msgRmv")?;
        // Check if there is an joined room
        chat_ws_tools::check_is_joined_room(self.room_id)?;
        // Check if there is a block on sending messages
        chat_ws_tools::check_is_blocked(self.is_blocked)?;

        let user_id = self.user_id;
        // Spawn an async task.
        let addr = ctx.address();
        let assistant = self.assistant.clone();
        // Start an additional asynchronous task.
        actix_web::rt::spawn(async move {
            // Delete a user's message in a chat.
            let result = assistant.execute_delete_chat_message(msg_rmv, user_id).await;
            if let Err(err) = result {
                return addr.do_send(AsyncResultError(err.status, err.code.to_string(), err.message.to_string()));
            }
            let opt_chat_message = result.unwrap();
            if opt_chat_message.is_none() {
                let message = format!("{}; id: {}, user_id: {}", err::MSG_CHAT_MESSAGE_NOT_FOUND, msg_rmv, user_id);
                return addr.do_send(AsyncResultError(404, code_to_str(StatusCode::NOT_FOUND), message.to_string()));
            }
            let text = to_string(&MsgRmvEWS { msg_rmv }).unwrap();
            // Send the "AsyncResultSendText" command for execution.
            addr.do_send(AsyncResultSendText(text));
        });
        Ok(())
    }

    pub fn handle_ews_name(&mut self, new_name: &str, ctx: &mut ws::WebsocketContext<Self>) -> Result<(), ErrEWS> {
        debug!("handle_ews_name() new_name: {}", new_name);
        // Check if this field is not empty
        chat_ws_tools::check_is_not_empty(&new_name, "name")?;
        // Check if there was a join to this room.
        if self.room_id != i32::default() {
            return Err(chat_ws_tools::get_err409(err::MSG_THERE_WAS_ALREADY_JOIN_TO_ROOM));
        }

        let user_name = self.user_name.clone();
        if new_name.len() > 0 && !user_name.eq(&new_name) {
            self.user_name = new_name.to_owned().clone();
        }
        let name = self.user_name.clone();
        ctx.text(to_string(&NameEWS { name }).unwrap());
        Ok(())
    }
}

// ** - **

// Adding functionality for processing parameters.
impl ChatWsSessionPrm for ChatWsSession {
    fn prm_room_id(&self) -> i32 {
        return self.room_id;
    }
    fn prm_is_blocked(&self) -> bool {
        return self.is_blocked;
    }
    fn prm_is_owner(&self) -> bool {
        return self.is_owner;
    }
    fn prm_issue_system_async<M: BrokerMsg>(&self, msg: M) {
        self.issue_system_async(msg);
    }
}

// * * * * Handler for asynchronous response to the "error" command. * * * *

struct AsyncResultError(
    u16,    // err
    String, // code
    String, // message
);

impl Message for AsyncResultError {
    type Result = ();
}

impl Handler<AsyncResultError> for ChatWsSession {
    type Result = ();

    fn handle(&mut self, msg: AsyncResultError, ctx: &mut Self::Context) {
        debug!("handle<AsyncResultError>() err: {}, code: {}, message: {}", msg.0, &msg.1, &msg.2);
        #[rustfmt::skip]
        ctx.text(to_string(&ErrEWS { err: msg.0, code: msg.1, message: msg.2 }).unwrap());
    }
}

// * * * * Handler for asynchronous response to the "JoinEWS" event * * * *

struct AsyncResultEwsJoin(
    i32,    // room_id
    i32,    // user_id
    String, // user_name
    bool,   // is_owner
    bool,   // is_blocked
);

impl Message for AsyncResultEwsJoin {
    type Result = ();
}

impl Handler<AsyncResultEwsJoin> for ChatWsSession {
    type Result = ();

    fn handle(&mut self, msg: AsyncResultEwsJoin, ctx: &mut Self::Context) {
        // If there is a room name, then leave it.
        if self.room_id > i32::default() {
            // Send message about "leave"
            let _ = self.handle_ews_leave(ctx);
        }
        let AsyncResultEwsJoin(room_id, user_id, user_name, is_owner, is_blocked) = msg;

        self.user_id = user_id;
        self.user_name = user_name.clone();
        self.is_owner = is_owner;
        self.is_blocked = is_blocked;

        // Then send a join message for the new room
        let join_room_srv = JoinRoom(room_id, self.user_name.clone(), ctx.address().recipient());
        // Send the "JoinRoom" command to the server.
        ChatWsServer::from_registry()
            .send(join_room_srv)
            .into_actor(self)
            .then(move |res, act_self, ctx| {
                if let Ok((id, count)) = res {
                    act_self.id = id;
                    act_self.room_id = room_id;
                    if log_enabled!(Debug) {
                        let s1 = format!("is_owner: {is_owner}, is_blocked: {is_blocked}");
                        debug!("handler<AsyncResultEwsJoin>() room_id:{room_id}, user_name: {user_name}, count:{count}, {s1}");
                    }
                    let is_owner = Some(is_owner);
                    let is_blocked = Some(is_blocked);
                    #[rustfmt::skip]
                    ctx.text(to_string(&JoinEWS { join: room_id, member: user_name, count, is_owner, is_blocked }).unwrap());
                }
                fut::ready(())
            })
            .wait(ctx);
    }
}

// * * * * Handler for asynchronous response to the "SendText" event * * * *

struct AsyncResultSendText(
    String, // text
);

impl Message for AsyncResultSendText {
    type Result = ();
}

impl Handler<AsyncResultSendText> for ChatWsSession {
    type Result = ();

    fn handle(&mut self, info: AsyncResultSendText, _ctx: &mut Self::Context) {
        // issue_async comes from having the `BrokerIssue` trait in scope.
        self.issue_system_async(SendMessage(self.room_id, info.0));
    }
}

// * * * * Handler for asynchronous response to the "BlockClient" event * * * *

struct AsyncResultBlockClient(
    i32,    // room_id
    bool,   // is_block
    String, // blocked_name
);

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

// **** ChatWsSession implementation "Handler<CommandSrv>" ****

impl Handler<CommandSrv> for ChatWsSession {
    type Result = MessageResult<CommandSrv>;

    fn handle(&mut self, command: CommandSrv, ctx: &mut Self::Context) -> Self::Result {
        match command {
            CommandSrv::Block(block) => self.handle_block_client(block, ctx),
            CommandSrv::Chat(chat_msg) => self.handle_chat_message_ssn(chat_msg, ctx),
            CommandSrv::CloseAndStop(reason) => self.handle_close_and_stop(reason, ctx),
        }

        MessageResult(())
    }
}

impl ChatWsSession {
    // Handler for "CommandSrv::Block(BlockSsn)".
    fn handle_block_client(&mut self, block: BlockSsn, ctx: &mut <Self as actix::Actor>::Context) {
        let BlockSsn(is_block, is_in_chat) = block;
        self.is_blocked = is_block;
        let user_name = self.user_name.clone();
        #[rustfmt::skip]
        let str = if is_block {
            to_string(&BlockEWS { block: user_name, is_in_chat }).unwrap()
        } else {
            to_string(&UnblockEWS { unblock: user_name, is_in_chat }).unwrap()
        };
        debug!("handler<CommandSrv::Block>() is_block: {is_block}, str: {str}");
        ctx.text(str);
    }

    // Handler for "CommandSrv::Chat(ChatMessageSsn)".
    fn handle_chat_message_ssn(&mut self, chat_msg: ChatMsgSsn, ctx: &mut ws::WebsocketContext<Self>) {
        ctx.text(chat_msg.0);
    }

    // Handler for "CommandSrv::CloseAndStop(Option<CloseReason>)".
    fn handle_close_and_stop(&mut self, reason: Option<CloseReason>, ctx: &mut ws::WebsocketContext<Self>) {
        debug!("handler<CommandSrv::Close>: ctx.close(reason); ctx.stop();");
        ctx.close(reason);
        ctx.stop();
    }
}
