use log::{debug, log_enabled, Level::Debug};

use actix::prelude::*;
use actix_broker::BrokerIssue;
// use actix_web::web;
use actix_web_actors::ws;
use chrono::{SecondsFormat /*Utc*/};
use serde_json::to_string;

use crate::chats::chat_event_ws::{
    BlockEWS, CountEWS, EWSType, EchoEWS, ErrEWS, EventWS, /*MsgCutEWS,*/ MsgEWS, /*MsgPutEWS,*/
    NameEWS, UnblockEWS,
};
use crate::chats::chat_message::{
    BlockClient, /*BlockSsn,*/ ChatMsgSsn, CommandSrv, CountMembers, JoinRoom, LeaveRoom,
    /*SaveMessageResult,*/ SendMessage,
};
use crate::chats::chat_ws_assistant::ChatWsAssistant;
use crate::chats::chat_ws_server::ChatWsServer;

pub const PARAMETER_NOT_DEFINED: &str = "parameter not defined";
pub const THERE_WAS_ALREADY_JOIN_TO_ROOM: &str = "There was already a \"join\" to the room";
pub const USERS_MSG_NOT_FOUND: &str = "Current user's message not found.";
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
        debug!("WEBSOCKET MESSAGE: {msg:?}");
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
        // Check if there is an joined room
        self.check_is_joined_room()?;
        // Check if there is a block on sending messages
        self.check_is_blocked()?;
        // Check if the member has a name or is anonymous
        self.check_is_name()?;
        Ok(())
    }
    // Checking the possibility of blocking a user.
    fn check_possibility_blocking(&self) -> Result<(), String> {
        // Check if there is an joined room
        self.check_is_joined_room()?;
        // Check if the member has a name or is anonymous
        self.check_is_name()?;
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
            debug!("WEBSOCKET: Error: {:?} msg: \"{}\"", err, msg);
            ctx.text(to_string(&ErrEWS { err }).unwrap());
            return;
        }
        let event = res_event.unwrap();

        match event.ews_type() {
            EWSType::Echo => {
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
                if let Err(err) = self.handle_ews_block(&block, true, ctx) {
                    ctx.text(to_string(&ErrEWS { err }).unwrap());
                }
            }
            EWSType::Count => {
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
            EWSType::Name => {
                let new_name = event.get_string("name").unwrap_or("".to_owned());
                // For an authorized user, id and name are defined.
                let id = self.user_id;
                let user_name = self.user_name.clone();
                if new_name.len() > 0 && (user_name.len() == 0 || !user_name.eq(&new_name)) {
                    self.user_name = new_name.clone();
                }
                let name = self.user_name.clone();
                ctx.text(to_string(&NameEWS { name, id }).unwrap());
            }
            _ => {}
        }
    }

    // ** Blocking clients in a room by name. (Session -> Server) **
    pub fn handle_ews_block(
        &self,
        client_name: &str,
        is_block: bool,
        ctx: &mut ws::WebsocketContext<Self>,
    ) -> Result<(), String> {
        // eprintln!("handle_ews_block(client_name: {client_name}, is_block: {is_block});");
        // Check if this field is required
        self.check_is_required_string(client_name, "block")?;
        #[rustfmt::skip]
        eprintln!("handle_ews_block(); .user_id: {}, .user_name: {}", self.user_id, self.user_name);
        // Checking the possibility of blocking a user.
        self.check_possibility_blocking()?;

        let room_id = self.room_id;
        let user_id = self.user_id;
        let block_name = client_name.to_string();
        // Spawn an async task.
        let addr = ctx.address();
        let assistant = self.assistant.clone();
        // Start an additional asynchronous task.
        actix_web::rt::spawn(async move {
            // Perform blocking/unblocking of a user.
            let result = assistant.execute_block_user(is_block, user_id, None, Some(block_name)).await;
            if let Err(err) = result {
                #[rustfmt::skip]
                eprintln!("handle_ews_block(); err1: {}", err.to_string());
                return addr.do_send(AsyncResultError(err.to_string()));
            }
            let opt_blocked_user = result.unwrap();
            if opt_blocked_user.is_none() {
                eprintln!("handle_ews_block(); err2: {SPECIFIED_USER_NOT_FOUND}");
                return addr.do_send(AsyncResultError(SPECIFIED_USER_NOT_FOUND.to_owned()));
            }
            let blocked_user = opt_blocked_user.unwrap();
            let blocked_name = blocked_user.blocked_nickname.clone();
            #[rustfmt::skip]
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
                let user_name = profile.nickname.clone();

                // Check the stream ID (room_id) and get the user_id of the stream owner.
                // TODO??
                let is_owner = profile.user_id == 18;

                // Send the "AsyncResultEwsJoin" command for execution.
                addr.do_send(AsyncResultEwsJoin(room_id, user_id, user_name, is_owner));
            });
        } else {
            // Send the "AsyncResultEwsJoin" command for execution.
            #[rustfmt::skip]
            ctx.address()
                .do_send(AsyncResultEwsJoin(room_id, i32::default(), self.user_name.clone(), false));
        }
        Ok(())
    }

    // * Leave the client from the chat room. (Session -> Server) *
    pub fn handle_ews_leave(&mut self, ctx: &mut ws::WebsocketContext<Self>) -> Result<(), String> {
        // Check if there is an joined room
        self.check_is_joined_room()?;

        // Send a message about leaving the room.
        let leave_room_srv = LeaveRoom(self.room_id, self.id, self.user_name.clone());
        // issue_sync comes from having the `BrokerIssue` trait in scope.
        self.issue_system_sync(leave_room_srv, ctx);
        // Reset room name.
        self.room_id = i32::default();
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

        // Spawn an async task.
        let addr = ctx.address();
        let assistant = self.assistant.clone();
        // Start an additional asynchronous task.
        actix_web::rt::spawn(async move {
            // Check the token for correctness and get the user profile.
            let result = assistant.execute_create_chat_message(stream_id, user_id, &msg).await;
            if let Err(err) = result {
                return addr.do_send(AsyncResultError(err.to_string()));
            }
            let ch_msg = result.unwrap();
            let msg2 = ch_msg.msg.unwrap_or_default();
            let date = ch_msg.date_update.to_rfc3339_opts(SecondsFormat::Millis, true);
            // Send the "AsyncResultEwsMsg" command for execution.
            #[rustfmt::skip]
            addr.do_send(AsyncResultEwsMsg(msg2, ch_msg.id, date, ch_msg.is_changed, ch_msg.is_removed));
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
        // Spawn an async task.
        let addr = ctx.address();
        let assistant = self.assistant.clone();
        // Start an additional asynchronous task.
        actix_web::rt::spawn(async move {
            // Check the token for correctness and get the user profile.
            let result = assistant.execute_modify_chat_message(id, user_id, &msg_cut).await;
            if let Err(err) = result {
                return addr.do_send(AsyncResultError(err.to_string()));
            }
            let opt_chat_message = result.unwrap();
            if opt_chat_message.is_none() {
                return addr.do_send(AsyncResultError(format!("{} (msg_id: {})", USERS_MSG_NOT_FOUND, id)));
            }
            let ch_msg = opt_chat_message.unwrap();
            let msg2 = ch_msg.msg.unwrap_or_default();
            let date = ch_msg.date_update.to_rfc3339_opts(SecondsFormat::Millis, true);
            // Send the "AsyncResultEwsMsg" command for execution.
            #[rustfmt::skip]
            addr.do_send(AsyncResultEwsMsg(msg2, ch_msg.id, date, ch_msg.is_changed, ch_msg.is_removed));
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
        // Spawn an async task.
        let addr = ctx.address();
        let assistant = self.assistant.clone();
        // Start an additional asynchronous task.
        actix_web::rt::spawn(async move {
            // Check the token for correctness and get the user profile.
            let result = assistant.execute_modify_chat_message(id, user_id, &msg_put).await;
            if let Err(err) = result {
                return addr.do_send(AsyncResultError(err.to_string()));
            }
            let opt_chat_message = result.unwrap();
            if opt_chat_message.is_none() {
                return addr.do_send(AsyncResultError(format!("{} (msg_id: {})", USERS_MSG_NOT_FOUND, id)));
            }
            let ch_msg = opt_chat_message.unwrap();
            let msg2 = ch_msg.msg.unwrap_or_default();
            let date = ch_msg.date_update.to_rfc3339_opts(SecondsFormat::Millis, true);
            // Send the "AsyncResultEwsMsg" command for execution.
            #[rustfmt::skip]
            addr.do_send(AsyncResultEwsMsg(msg2, ch_msg.id, date, ch_msg.is_changed, ch_msg.is_removed));
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
        let room_id = msg.0;
        self.user_id = msg.1;
        self.user_name = msg.2;
        self.is_owner = msg.3;

        // Then send a join message for the new room
        let join_room_srv = JoinRoom(room_id, self.user_name.clone(), ctx.address().recipient());
        // Send the "JoinRoom" command to the server.
        ChatWsServer::from_registry()
            .send(join_room_srv)
            .into_actor(self)
            .then(move |res, act, _ctx| {
                if let Ok(id) = res {
                    act.id = id;
                    act.room_id = room_id;
                }
                fut::ready(())
            })
            .wait(ctx);
    }
}

// * * * * Handler for asynchronous response to the "MsgEWS" event * * * *

struct AsyncResultEwsMsg(
    String, // message
    i32,    // message_id
    String, // message date
    bool,   // edit flag
    bool,   // delete flag
);

impl Message for AsyncResultEwsMsg {
    type Result = ();
}

impl Handler<AsyncResultEwsMsg> for ChatWsSession {
    type Result = ();

    fn handle(&mut self, info: AsyncResultEwsMsg, _ctx: &mut Self::Context) {
        let msg = info.0;
        let id = info.1;
        let member = self.user_name.clone();
        let date = info.2;
        let is_edt = info.3;
        let is_rmv = info.4;
        #[rustfmt::skip]
        let msg_str = to_string(&MsgEWS { msg, id, member, date, is_edt, is_rmv }).unwrap();

        // issue_async comes from having the `BrokerIssue` trait in scope.
        self.issue_system_async(SendMessage(self.room_id, msg_str));
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
        #[rustfmt::skip]
        eprintln!("handle<AsyncResultBlockClient>(); is_block: {}, blocked_name: {}", is_block, &blocked_name);

        let block_client = BlockClient(room_id, blocked_name.clone(), is_block);

        ChatWsServer::from_registry()
            .send(block_client)
            .into_actor(self)
            .then(move |res, _act, ctx| {
                eprintln!("handle<AsyncResultBlockClient>(); .send(block_client)");
                if let Ok(is_in_chat) = res {
                    eprintln!("handle<AsyncResultBlockClient>(); is_in_chat: {is_in_chat}");
                    #[rustfmt::skip]
                    let str = if is_block {
                        to_string(&BlockEWS { block: blocked_name, is_in_chat }).unwrap()
                    } else {
                        to_string(&UnblockEWS { unblock: blocked_name, is_in_chat }).unwrap()
                    };
                    eprintln!("handler<AsyncResultBlockClient>; ctx.text({})", &str);
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
            // CommandSrv::Block(blocking) => self.handle_block_client(blocking, ctx),
            CommandSrv::Chat(chat_msg) => self.handle_chat_message_ssn(chat_msg, ctx),
            _ => {}
        }

        MessageResult(())
    }
}

impl ChatWsSession {
    // Handler for "CommandSrv::Block(BlockSsn)".
    /*fn handle_block_client(&mut self, blocking: BlockSsn, ctx: &mut <Self as actix::Actor>::Context) {
        let BlockSsn(is_blocked) = blocking;
        self.is_blocked = is_blocked;
        let client_name = self.name.clone().unwrap_or("".to_owned());
        #[rustfmt::skip]
        let str = if is_blocked {
            to_string(& BlockEWS { block: client_name, count: 1 }).unwrap()
        } else {
            to_string(& UnblockEWS { unblock: client_name, count: 1 }).unwrap()
        };
        ctx.text(str);
    }*/

    // Handler for "CommandSrv::Chat(ChatMessageSsn)".
    fn handle_chat_message_ssn(&mut self, chat_msg: ChatMsgSsn, ctx: &mut ws::WebsocketContext<Self>) {
        ctx.text(chat_msg.0);
    }
}

// ****  __  ****

// **** ChatWsSession implementation "Handler<SaveMessageResult>" ****

/*impl Handler<SaveMessageResult> for ChatWsSession {
    type Result = MessageResult<SaveMessageResult>;

    fn handle(&mut self, msg_res: SaveMessageResult, _ctx: &mut Self::Context) -> Self::Result {
        debug!("#!_handler<SaveMessageResult>() msg_res: {:?}", msg_res.0);
        MessageResult(msg_res.0)
    }
}*/
