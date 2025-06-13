use log::{debug, log_enabled, Level::Debug};

use actix::prelude::*;
use actix_broker::BrokerIssue;
// use actix_web::web;
use actix_web_actors::ws;
use chrono::{SecondsFormat /*Utc*/};
use serde_json::to_string;

use crate::chats::{
    chat_event_ws::{
        BlockEWS, CountEWS, EWSType, EchoEWS, ErrEWS, EventWS, JoinEWS, MsgEWS, MsgRmvEWS, NameEWS, UnblockEWS,
    },
    chat_message::{BlockClient, BlockSsn, ChatMsgSsn, CommandSrv, CountMembers, JoinRoom, LeaveRoom, SendMessage},
    chat_ws_assistant::ChatWsAssistant,
    chat_ws_server::ChatWsServer,
};
use crate::errors::AppError;
use crate::settings::err;

pub const PARAMETER_NOT_DEFINED: &str = "parameter not defined";
pub const THERE_WAS_ALREADY_JOIN_TO_ROOM: &str = "There was already a \"join\" to the room";
pub const SPECIFIED_USER_NOT_FOUND: &str = "The specified user was not found.";
pub const STREAM_WITH_SPECIFIED_ID_NOT_FOUND: &str = "Stream with the specified id not found.";
pub const STREAM_NOT_ACTIVE: &str = "This stream is not active.";
// 404 Not Found - Stream not found.
pub const MSG_CHAT_MESSAGE_NOT_FOUND: &str = "chat_message_not_found";

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
            #[rustfmt::skip]
            let user_str = format!("user_id: {}, user_name: \"{}\", id: {}", self.user_id, &self.user_name, self.id);
            debug!("Session opened for user({}) in room_id {}.", user_str, self.room_id);
        }
    }
    // Called after an actor is stopped.
    fn stopped(&mut self, _ctx: &mut Self::Context) {
        if log_enabled!(Debug) {
            #[rustfmt::skip]
            let user_str = format!("user_id: {}, user_name: \"{}\", id: {}", self.user_id, &self.user_name, self.id);
            debug!("Session closed for user({}) in room_id {}.", user_str, self.room_id);
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
        debug!("websocket message: {msg:?}");
        match msg {
            ws::Message::Text(text) => {
                // Handle socket text messages.
                self.handle_text_messages(text.trim(), ctx);
            }
            ws::Message::Close(reason) => {
                // Send message about "leave"
                let _ = self.handle_ews_leave(ctx);
                ctx.close(reason);
                ctx.stop();
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
    // Check if this field is required
    #[rustfmt::skip]
    fn check_is_required_string(&self, value: &str, name: &str) -> Result<(), String> {
        if value.len() == 0 { Err(format!("\"{}\" {}", name, PARAMETER_NOT_DEFINED)) } else { Ok(()) }
    }
    #[rustfmt::skip]
    fn check_is_required_i32(&self, value: i32, name: &str) -> Result<(), String> {
        if value <= i32::default() { Err(format!("\"{}\" {}", name, PARAMETER_NOT_DEFINED)) } else { Ok(()) }
    }
    // Check if there is an joined room
    #[rustfmt::skip]
    fn check_is_joined_room(&self) -> Result<(), String> {
        if self.room_id <= i32::default() { Err("There was no \"join\" command.".to_owned()) } else { Ok(()) }
    }
    // Check if there is a block on sending messages
    #[rustfmt::skip]
    fn check_is_blocked(&self) -> Result<(), String> {
        if self.is_blocked { Err("There is a block on sending messages.".to_owned()) } else { Ok(()) }
    }
    // Check if the member has a name or is anonymous
    fn check_is_name(&self) -> Result<(), String> {
        if self.user_name.len() == 0 {
            return Err("There was no \"name\" command.".to_owned());
        }
        if self.user_id <= i32::default() {
            return Err("User ID not specified.".to_owned());
        }
        Ok(())
    }
    // Checking the possibility of sending a message.
    fn check_possibility_sending_message(&self) -> Result<(), String> {
        // Check if the member has a name or is anonymous
        self.check_is_name()?;
        // Check if there is an joined room
        self.check_is_joined_room()?;
        // Check if there is a block on sending messages
        self.check_is_blocked()?;
        Ok(())
    }
    // Checking the possibility of blocking a user.
    fn check_possibility_blocking(&self) -> Result<(), String> {
        // Check if the member has a name or is anonymous
        self.check_is_name()?;
        // Check if there is an joined room
        self.check_is_joined_room()?;
        // Check if the user is the owner of the stream.
        if !self.is_owner {
            return Err("Stream owner rights are missing.".to_owned());
        }
        Ok(())
    }

    /** Handle socket text messages. */
    fn handle_text_messages(&mut self, msg: &str, ctx: &mut ws::WebsocketContext<Self>) {
        // Parse input data of ws event.
        let res_event = EventWS::parsing(msg);
        if let Err(err) = res_event {
            debug!("websocket error: {:?} msg: \"{}\"", err, msg);
            ctx.text(to_string(&ErrEWS { err }).unwrap());
            return;
        }
        let event = res_event.unwrap();

        match event.ews_type() {
            EWSType::Echo => {
                // {"echo": "text1"}
                let echo = event.get_string("echo").unwrap_or("".to_owned());
                // Check if this field is required
                if let Err(err) = self.check_is_required_string(&echo, "echo") {
                    ctx.text(to_string(&ErrEWS { err }).unwrap());
                } else {
                    ctx.text(to_string(&EchoEWS { echo }).unwrap());
                }
            }
            EWSType::Block => {
                // {"block": "User2"}
                let block = event.get_string("block").unwrap_or("".to_owned());
                if let Err(err) = self.handle_ews_block_add_task(&block, true, ctx) {
                    ctx.text(to_string(&ErrEWS { err }).unwrap());
                }
            }
            EWSType::Count => {
                // {"count": -1}
                if let Err(err) = self.handle_ews_count(ctx) {
                    ctx.text(to_string(&ErrEWS { err }).unwrap());
                }
            }
            EWSType::Join => {
                // {"join": 1}
                let room_id = event.get_i32("join").unwrap_or_default();
                let access = event.get_string("access").unwrap_or("".to_owned());
                if let Err(err) = self.handle_ews_join_add_task(room_id, &access, ctx) {
                    ctx.text(to_string(&ErrEWS { err }).unwrap());
                }
            }
            EWSType::Leave => {
                // {"leave": -1}
                if let Err(err) = self.handle_ews_leave(ctx) {
                    ctx.text(to_string(&ErrEWS { err }).unwrap());
                }
            }
            EWSType::Msg => {
                // {"msg":"text msg"}
                let msg = event.get_string("msg").unwrap_or_default();
                if let Err(err) = self.handle_ews_msg_add_task(&msg, ctx) {
                    ctx.text(to_string(&ErrEWS { err }).unwrap());
                }
            }
            EWSType::MsgCut => {
                // {"msgCut": "", "id": 1}
                let id = event.get_i32("id").unwrap_or_default();
                if let Err(err) = self.handle_ews_msg_cut_add_task(id, ctx) {
                    ctx.text(to_string(&ErrEWS { err }).unwrap());
                }
            }
            EWSType::MsgPut => {
                // {"msgPut": "modify msg", "id": 1}
                let msg_put = event.get_string("msgPut").unwrap_or_default();
                let id = event.get_i32("id").unwrap_or_default();
                if let Err(err) = self.handle_ews_msg_put_add_task(&msg_put, id, ctx) {
                    ctx.text(to_string(&ErrEWS { err }).unwrap());
                }
            }
            EWSType::MsgRmv => {
                // {"msgRmv": 1}
                let msg_rmv = event.get_i32("msgRmv").unwrap_or_default();
                if let Err(err) = self.handle_ews_msg_rmv_add_task(msg_rmv, ctx) {
                    ctx.text(to_string(&ErrEWS { err }).unwrap());
                }
            }
            EWSType::Name => {
                // {"name": "User1"}
                let new_name = event.get_string("name").unwrap_or("".to_owned());
                // Check if this field is required
                if let Err(err) = self.check_is_required_string(&new_name, "name") {
                    ctx.text(to_string(&ErrEWS { err }).unwrap());
                }
                // For an authorized user, id and name are defined.
                let id = self.user_id;
                let user_name = self.user_name.clone();
                if new_name.len() > 0 && (user_name.len() == 0 || !user_name.eq(&new_name)) {
                    self.user_name = new_name.clone();
                }
                let name = self.user_name.clone();
                ctx.text(to_string(&NameEWS { name, id }).unwrap());
            }
            EWSType::Unblock => {
                // {"unblock": "User2"}
                let block = event.get_string("unblock").unwrap_or("".to_owned());
                if let Err(err) = self.handle_ews_block_add_task(&block, false, ctx) {
                    ctx.text(to_string(&ErrEWS { err }).unwrap());
                }
            }
            _ => {}
        }
    }

    // ** Blocking clients in a room by name. (Session -> Server) **
    pub fn handle_ews_block_add_task(
        &self,
        user_name: &str,
        is_block: bool,
        ctx: &mut ws::WebsocketContext<Self>,
    ) -> Result<(), String> {
        debug!("handle_ews_block_add_task() user_name: {user_name}, is_block: {is_block}");
        let tag_name = if is_block { "block" } else { "unblock" };
        // Check if this field is required
        self.check_is_required_string(user_name, tag_name)?;
        // Checking the possibility of blocking a user.
        self.check_possibility_blocking()?;

        let room_id = self.room_id;
        let user_id = self.user_id;
        let block_name = user_name.to_string();
        // Spawn an async task.
        let addr = ctx.address();
        let assistant = self.assistant.clone();
        // Start an additional asynchronous task.
        actix_web::rt::spawn(async move {
            // Perform blocking/unblocking of a user.
            let result = assistant.execute_block_user(is_block, user_id, None, Some(block_name)).await;
            if let Err(err) = result {
                return addr.do_send(AsyncResultError(err.to_string()));
            }
            let opt_blocked_user = result.unwrap();
            if opt_blocked_user.is_none() {
                return addr.do_send(AsyncResultError(SPECIFIED_USER_NOT_FOUND.to_owned()));
            }
            let blocked_user = opt_blocked_user.unwrap();
            let blocked_name = blocked_user.blocked_nickname.clone();
            addr.do_send(AsyncResultBlockClient(room_id, is_block, blocked_name));
        });
        Ok(())
    }

    // ** Count of clients in the room. (Session -> Server) **
    pub fn handle_ews_count(&mut self, ctx: &mut ws::WebsocketContext<Self>) -> Result<(), String> {
        // Check if there is an joined room
        self.check_is_joined_room()?;
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
    pub fn handle_ews_join_add_task(
        &mut self,
        room_id: i32,
        access: &str,
        ctx: &mut ws::WebsocketContext<Self>,
    ) -> Result<(), String> {
        #[rustfmt::skip]
        debug!("handle_ews_join_add_task() room_id: {room_id}, access.len(): {}", access.len());
        // Check if this field is required
        self.check_is_required_i32(room_id, "join")?;
        // Check if there was a join to this room.
        if self.room_id == room_id {
            return Err(format!("{} {}.", THERE_WAS_ALREADY_JOIN_TO_ROOM, room_id));
        }
        // If "access" is specified, then get the user profile.
        if access.len() > 0 {
            // Decode the token. And unpack the two parameters from the token.
            let (user_id, num_token) = self.assistant.decode_and_verify_token(access)?;
            // Spawn an async task.
            let addr = ctx.address();
            let assistant = self.assistant.clone();
            // Start an additional asynchronous task.
            actix_web::rt::spawn(async move {
                // Check the token for correctness and get the user profile.
                let result = assistant.check_num_token_and_get_profile(user_id, num_token).await;

                if let Err(err) = result {
                    return addr.do_send(AsyncResultError(err.to_string()));
                }
                let profile = result.unwrap();
                let user_id = profile.user_id.clone();
                let nickname = profile.nickname.clone();
                let user_name = nickname.clone();
                // Get chat access information.
                let result2 = assistant.get_chat_access(room_id, user_id).await;

                if let Err(err) = result2 {
                    return addr.do_send(AsyncResultError(err.to_string()));
                }
                let opt_chat_access = result2.unwrap();
                // If the stream with id = room_id is not found, then an error occurs.
                if opt_chat_access.is_none() {
                    return addr.do_send(AsyncResultError(STREAM_WITH_SPECIFIED_ID_NOT_FOUND.to_owned()));
                }
                let chat_access = opt_chat_access.unwrap();
                // Check stream activity. (live: "state" IN ('preparing', 'started', 'paused'))
                if !chat_access.stream_live {
                    return addr.do_send(AsyncResultError(STREAM_NOT_ACTIVE.to_owned()));
                }
                // Determine if a user is the owner of a chat.
                let is_owner = profile.user_id == chat_access.stream_owner;
                // Get the "block" value for the given user.
                let is_blocked = chat_access.is_blocked;

                debug!("handle_ews_join_add_task() room_id:{room_id}, user_name:{nickname}, is_owner:{is_owner}, is_blocked:{is_blocked}");
                // Send the "AsyncResultEwsJoin" command for execution.
                addr.do_send(AsyncResultEwsJoin(room_id, user_id, user_name, is_owner, is_blocked));
            });
        } else {
            debug!("handle_ews_join_add_task() room_id: {room_id}, user_name: , is_owner: false, is_blocked: true");
            // Send the "AsyncResultEwsJoin" command for execution.
            #[rustfmt::skip]
            ctx.address()
                .do_send(AsyncResultEwsJoin(room_id, i32::default(), self.user_name.clone(), false, true));
        }
        Ok(())
    }

    // * Leave the client from the chat room. (Session -> Server) *
    pub fn handle_ews_leave(&mut self, ctx: &mut ws::WebsocketContext<Self>) -> Result<(), String> {
        #[rustfmt::skip]
        debug!("handle_ews_leave() room_id: {}, user_name: {}", self.room_id, self.user_name);
        // Check if there is an joined room
        self.check_is_joined_room()?;

        // Send a message about leaving the room.
        let leave_room_srv = LeaveRoom(self.room_id, self.id, self.user_name.clone());
        // issue_sync comes from having the `BrokerIssue` trait in scope.
        self.issue_system_sync(leave_room_srv, ctx);
        // Reset room name.
        self.room_id = i32::default();
        self.is_owner = false;
        self.is_blocked = false;
        Ok(())
    }

    // * Send a text message to all clients in the room. (Server -> Session) *
    pub fn handle_ews_msg_add_task(&self, msg: &str, ctx: &mut ws::WebsocketContext<Self>) -> Result<(), String> {
        let msg = msg.to_owned();
        // Check if this field is required
        self.check_is_required_string(&msg, "msg")?;
        // Checking the possibility of sending a message.
        self.check_possibility_sending_message()?;

        // Get room (stream) ID and user ID.
        let stream_id = self.room_id;
        let user_id = self.user_id;
        let member = self.user_name.clone();
        // Spawn an async task.
        let addr = ctx.address();
        let assistant = self.assistant.clone();
        // Start an additional asynchronous task.
        actix_web::rt::spawn(async move {
            // Create a new user message in the chat.
            let result = assistant.execute_create_chat_message(stream_id, user_id, &msg).await;
            if let Err(err) = result {
                return addr.do_send(AsyncResultError(err.to_string()));
            }
            let opt_chat_message = result.unwrap();
            if opt_chat_message.is_none() {
                let message = format!("{}: stream_id: {}", err::MSG_STREAM_NOT_FOUND, stream_id);
                return addr.do_send(AsyncResultError(AppError::not_found404(&message).to_string()));
            }
            let ch_msg = opt_chat_message.unwrap();
            // Send the "AsyncResultSendText" command for execution.
            addr.do_send(AsyncResultSendText(
                to_string(&MsgEWS {
                    msg: ch_msg.msg.unwrap_or_default(),
                    id: ch_msg.id,
                    member,
                    date: ch_msg.date_update.to_rfc3339_opts(SecondsFormat::Millis, true),
                    is_edt: ch_msg.is_changed,
                    is_rmv: ch_msg.is_removed,
                })
                .unwrap(),
            ));
        });
        Ok(())
    }

    // * Send a delete message to all chat members. (Server -> Session) *
    pub fn handle_ews_msg_cut_add_task(&self, id: i32, ctx: &mut ws::WebsocketContext<Self>) -> Result<(), String> {
        let msg_cut = "".to_owned();
        // Check if this field is required
        self.check_is_required_i32(id, "id")?;
        // Checking the possibility of sending a message.
        self.check_possibility_sending_message()?;

        let user_id = self.user_id;
        let opt_user_id = Some(self.user_id);
        let member = self.user_name.clone();
        // Spawn an async task.
        let addr = ctx.address();
        let assistant = self.assistant.clone();
        // Start an additional asynchronous task.
        actix_web::rt::spawn(async move {
            // Change a user's message in a chat.
            let result = assistant.execute_modify_chat_message(id, opt_user_id, &msg_cut).await;
            if let Err(err) = result {
                return addr.do_send(AsyncResultError(err.to_string()));
            }
            let opt_chat_message = result.unwrap();
            if opt_chat_message.is_none() {
                let message = format!("{}; id: {}, user_id: {}", MSG_CHAT_MESSAGE_NOT_FOUND, id, user_id);
                return addr.do_send(AsyncResultError(AppError::not_found404(&message).to_string()));
            }
            let ch_msg = opt_chat_message.unwrap();
            // Send the "AsyncResultSendText" command for execution.
            addr.do_send(AsyncResultSendText(
                to_string(&MsgEWS {
                    msg: ch_msg.msg.unwrap_or_default(),
                    id: ch_msg.id,
                    member,
                    date: ch_msg.date_update.to_rfc3339_opts(SecondsFormat::Millis, true),
                    is_edt: ch_msg.is_changed,
                    is_rmv: ch_msg.is_removed,
                })
                .unwrap(),
            ));
        });
        Ok(())
    }
    // * Send a correction to the message to everyone in the chat. (Server -> Session) *
    pub fn handle_ews_msg_put_add_task(
        &self,
        msg_put: &str,
        id: i32,
        ctx: &mut ws::WebsocketContext<Self>,
    ) -> Result<(), String> {
        let msg_put = msg_put.to_owned();
        // Check if this field is required
        self.check_is_required_string(&msg_put, "msgPut")?;
        // Check if this field is required
        self.check_is_required_i32(id, "id")?;
        // Checking the possibility of sending a message.
        self.check_possibility_sending_message()?;

        let user_id = self.user_id;
        let opt_user_id = Some(self.user_id);
        let member = self.user_name.clone();
        // Spawn an async task.
        let addr = ctx.address();
        let assistant = self.assistant.clone();
        // Start an additional asynchronous task.
        actix_web::rt::spawn(async move {
            // Change a user's message in a chat.
            let result = assistant.execute_modify_chat_message(id, opt_user_id, &msg_put).await;
            if let Err(err) = result {
                return addr.do_send(AsyncResultError(err.to_string()));
            }
            let opt_chat_message = result.unwrap();
            if opt_chat_message.is_none() {
                let message = format!("{}; id: {}, user_id: {}", MSG_CHAT_MESSAGE_NOT_FOUND, id, user_id);
                return addr.do_send(AsyncResultError(AppError::not_found404(&message).to_string()));
            }
            let ch_msg = opt_chat_message.unwrap();
            // Send the "AsyncResultSendText" command for execution.
            addr.do_send(AsyncResultSendText(
                to_string(&MsgEWS {
                    msg: ch_msg.msg.unwrap_or_default(),
                    id: ch_msg.id,
                    member,
                    date: ch_msg.date_update.to_rfc3339_opts(SecondsFormat::Millis, true),
                    is_edt: ch_msg.is_changed,
                    is_rmv: ch_msg.is_removed,
                })
                .unwrap(),
            ));
        });
        Ok(())
    }
    // * Send a permanent deletion message to all chat members. *
    pub fn handle_ews_msg_rmv_add_task(
        &self,
        msg_rmv: i32,
        ctx: &mut ws::WebsocketContext<Self>,
    ) -> Result<(), String> {
        // Check if this field is required
        self.check_is_required_i32(msg_rmv, "msgRmv")?;
        // Checking the possibility of sending a message.
        self.check_possibility_sending_message()?;

        let user_id = self.user_id;
        let opt_user_id = Some(self.user_id);
        // Spawn an async task.
        let addr = ctx.address();
        let assistant = self.assistant.clone();
        // Start an additional asynchronous task.
        actix_web::rt::spawn(async move {
            // Delete a user's message in a chat.
            let result = assistant.execute_delete_chat_message(msg_rmv, opt_user_id).await;
            if let Err(err) = result {
                return addr.do_send(AsyncResultError(err.to_string()));
            }
            let opt_chat_message = result.unwrap();
            if opt_chat_message.is_none() {
                let message = format!("{}; id: {}, user_id: {}", MSG_CHAT_MESSAGE_NOT_FOUND, msg_rmv, user_id);
                return addr.do_send(AsyncResultError(AppError::not_found404(&message).to_string()));
            }
            let text = to_string(&MsgRmvEWS { msg_rmv }).unwrap();
            // Send the "AsyncResultSendText" command for execution.
            addr.do_send(AsyncResultSendText(text));
        });
        Ok(())
    }
}

// ** - **

// * * * * Handler for asynchronous response to the "error" command. * * * *

struct AsyncResultError(String);

impl Message for AsyncResultError {
    type Result = ();
}

impl Handler<AsyncResultError> for ChatWsSession {
    type Result = ();

    fn handle(&mut self, msg: AsyncResultError, ctx: &mut Self::Context) {
        debug!("handle<AsyncResultError>() msg.0: {}", &msg.0);
        ctx.text(to_string(&ErrEWS { err: msg.0 }).unwrap());
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
            .then(move |res, act, ctx| {
                if let Ok((id, count)) = res {
                    act.id = id;
                    act.room_id = room_id;
                    if log_enabled!(Debug) {
                        let s1 = format!("is_owner: {is_owner}, is_blocked: {is_blocked}");
                        debug!("handler<AsyncResultEwsJoin>() room_id:{room_id}, user_name: {user_name}, count:{count}, {s1}");
                    }
                    #[rustfmt::skip]
                    ctx.text(to_string(&JoinEWS {
                        join: room_id, member: user_name, count, is_owner: Some(is_owner), is_blocked: Some(is_blocked) }).unwrap());
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
        debug!("handler<CommandSrv::BlockSsn>() is_block: {is_block}, str: {str}");
        ctx.text(str);
    }

    // Handler for "CommandSrv::Chat(ChatMessageSsn)".
    fn handle_chat_message_ssn(&mut self, chat_msg: ChatMsgSsn, ctx: &mut ws::WebsocketContext<Self>) {
        ctx.text(chat_msg.0);
    }
}

// ****  __  ****
