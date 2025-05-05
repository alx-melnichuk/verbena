use actix::prelude::*;
use actix_broker::BrokerIssue;
// use actix_web::web;
use actix_web_actors::ws;
use chrono::{SecondsFormat /*Utc*/};
use serde_json::to_string;

use crate::chats::chat_event_ws::{
    /*BlockEWS,*/ CountEWS, EWSType, EchoEWS, ErrEWS, EventWS, /*MsgCutEWS,*/ MsgEWS,
    /*MsgPutEWS,*/ NameEWS, /*UnblockEWS,*/
};
use crate::chats::chat_message::{
    /*BlockClients, BlockSsn,*/ ChatMsgSsn, CommandSrv, CountMembers, JoinRoom, LeaveRoom,
    /*SaveMessageResult,*/ SendMessage,
};
use crate::chats::chat_message_models::{ChatMessage, CreateChatMessage};
use crate::chats::chat_ws_assistant::ChatWsAssistant;
use crate::chats::chat_ws_server::ChatWsServer;
// use crate::profiles::profile_models::Profile;
// use crate::settings::err;

pub const PARAMETER_NOT_DEFINED: &str = "parameter not defined";
pub const THERE_WAS_ALREADY_JOIN_TO_ROOM: &str = "There was already a \"join\" to the room";

// ** ChatWsSession **

pub struct ChatWsSession {
    id: u64,
    room_id: Option<i32>,
    user_id: Option<i32>,
    user_name: Option<String>,
    is_blocked: bool,
    assistant: ChatWsAssistant,
}

// ** ChatWsSession implementation "Actor" **

impl Actor for ChatWsSession {
    type Context = ws::WebsocketContext<Self>;
    // Called when an actor gets polled the first time.
    fn started(&mut self, _ctx: &mut Self::Context) {
        let user_id = self.user_id.clone().unwrap_or(-1);
        let user_name = self.user_name.clone().unwrap_or_default();
        let user_str = format!("(user_id: {}, user_name: \"{}\", id: {})", user_id, user_name, self.id);
        let room_id = self.room_id.clone().unwrap_or(-1);
        log::debug!("Session opened for user{} in room \"{}\".", user_str, room_id);
    }
    // Called after an actor is stopped.
    fn stopped(&mut self, _ctx: &mut Self::Context) {
        let user_id = self.user_id.clone().unwrap_or(-1);
        let user_name = self.user_name.clone().unwrap_or_default();
        let user_str = format!("(user_id: {}, user_name: \"{}\", id: {})", user_id, user_name, self.id);
        let room_id = self.room_id.clone().unwrap_or(-1);
        log::debug!("Session closed for user{} in room \"{}\".", user_str, room_id);
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
        log::debug!("WEBSOCKET MESSAGE: {msg:?}");
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
        room_id: Option<i32>,
        user_id: Option<i32>,
        user_name: Option<String>,
        is_blocked: bool,
        assistant: ChatWsAssistant,
    ) -> Self {
        ChatWsSession {
            id,
            room_id,
            user_id,
            user_name,
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
        if value < 0 { Err(format!("\"{}\" {}", name, PARAMETER_NOT_DEFINED)) } else { Ok(()) }
    }
    // Check if there is an joined room
    #[rustfmt::skip]
    fn check_is_joined_room(&self) -> Result<(), String> {
        if self.room_id.is_none() { Err("There was no \"join\" command.".to_owned()) } else { Ok(()) }
    }
    // // Check if there is a block on sending messages
    // fn check_is_blocked(&self) -> Result<(), String> {
    //     if self.is_blocked { Err("There is a block on sending messages.".to_owned()) } else { Ok(()) }
    // }
    // // Check if the member has a name or is anonymous
    // fn check_is_name(&self) -> Result<(), String> {
    //     let name = self.name.clone().unwrap_or("".to_owned());
    //     if name.len() == 0 { Err("There was no \"name\" command.".to_owned()) } else { Ok(()) }
    // }

    /** Handle socket text messages. */
    fn handle_text_messages(&mut self, msg: &str, ctx: &mut ws::WebsocketContext<Self>) {
        eprintln!("#!_WEBSOCKET: Text msg: \"{}\"", msg);
        log::debug!("WEBSOCKET: Text: msg: \"{}\"", msg);
        // Parse input data of ws event.
        let res_event = EventWS::parsing(msg);
        if let Err(err) = res_event {
            log::debug!("WEBSOCKET: Error: {:?} msg: \"{}\"", err, msg);
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
            EWSType::Count => {
                if let Err(err) = self.handle_ews_count(ctx) {
                    ctx.text(to_string(&ErrEWS { err }).unwrap());
                }
            }
            EWSType::Join => {
                // {"join": 1}
                let room_id = event.get_i32("join").unwrap_or(-1);
                let access = event.get_string("access").unwrap_or("".to_owned());
                if let Err(err) = self.handle_ews_join_ext_task(room_id, &access, ctx) {
                    ctx.text(to_string(&ErrEWS { err }).unwrap());
                }
            }
            EWSType::Leave => {
                if let Err(err) = self.handle_ews_leave(ctx) {
                    ctx.text(to_string(&ErrEWS { err }).unwrap());
                }
            }
            EWSType::Msg => {
                let msg = event.get_string("msg").unwrap_or("".to_string());
                if let Err(err) = self.handle_ews_msg_ext_task(&msg, ctx) {
                    ctx.text(to_string(&ErrEWS { err }).unwrap());
                }
            }
            EWSType::Name => {
                let new_name = event.get_string("name").unwrap_or("".to_owned());
                // For an authorized user, id and name are defined.
                let id = self.user_id.clone().unwrap_or_default();
                let user_name = self.user_name.clone().unwrap_or_default();
                if new_name.len() > 0 && (user_name.len() == 0 || !user_name.eq(&new_name)) {
                    self.user_name = Some(new_name);
                }
                let name = self.user_name.clone().unwrap_or("".to_owned());
                ctx.text(to_string(&NameEWS { id, name }).unwrap());
            }
            _ => {}
        }
    }

    // // * Blocking clients in a room by name. (Session -> Server) *
    // pub fn blocking_clients(&self,client_name: &str,is_blocked: bool,ctx: &mut ws::WebsocketContext<Self>,) -> Result<(), String> {    }

    // ** Count of clients in the room. (Session -> Server) **
    pub fn handle_ews_count(&mut self, ctx: &mut ws::WebsocketContext<Self>) -> Result<(), String> {
        // Check if there is an joined room
        self.check_is_joined_room()?;
        let count_members = CountMembers(self.room_id.unwrap_or_default());

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
    pub fn handle_ews_join_ext_task(
        &mut self,
        room_id: i32,
        access: &str,
        ctx: &mut ws::WebsocketContext<Self>,
    ) -> Result<(), String> {
        // Check if this field is required
        self.check_is_required_i32(room_id, "join")?;
        // Check if there was a join to this room.
        if self.room_id.unwrap_or_default() == room_id {
            return Err(format!("{} {}.", THERE_WAS_ALREADY_JOIN_TO_ROOM, room_id));
        }
        // If "access" is specified, then get the user profile.
        if access.len() > 0 {
            // Decode the token. And unpack the two parameters from the token.
            let (user_id, num_token) = self.assistant.decode_and_verify_token(access)?;
            // Spawn an async task.
            let addr = ctx.address();
            // Get a clone of the assistant.
            let assistant = self.assistant.clone();
            // Start an additional asynchronous task.
            actix_web::rt::spawn(async move {
                // Check the token for correctness and get the user profile.
                let result = assistant.check_num_token_and_get_profile(user_id, num_token).await;
                if let Err(err) = result {
                    return addr.do_send(AsyncResultError(err.to_string()));
                }
                let profile = result.unwrap();
                let user_id = Some(profile.user_id);
                let user_name = Some(profile.nickname.clone());
                // Send the "AsyncResultEwsJoin" command for execution.
                addr.do_send(AsyncResultEwsJoin(room_id, user_id, user_name));
            });
        } else {
            // Send the "AsyncResultEwsJoin" command for execution.
            ctx.address().do_send(AsyncResultEwsJoin(room_id, None, self.user_name.clone()));
        }
        Ok(())
    }

    // * Leave the client from the chat room. (Session -> Server) *
    pub fn handle_ews_leave(&mut self, ctx: &mut ws::WebsocketContext<Self>) -> Result<(), String> {
        // Check if there is an joined room
        self.check_is_joined_room()?;

        let room_id = self.room_id.unwrap_or_default();
        // Send a message about leaving the room.
        let leave_room_srv = LeaveRoom(room_id, self.id, self.user_name.clone());
        // issue_sync comes from having the `BrokerIssue` trait in scope.
        self.issue_system_sync(leave_room_srv, ctx);
        // Reset room name.
        self.room_id = None;
        Ok(())
    }

    // * Send a text message to all clients in the room. (Server -> Session) *
    pub fn handle_ews_msg_ext_task(&self, msg: &str, ctx: &mut ws::WebsocketContext<Self>) -> Result<(), String> {
        let msg = msg.to_owned();
        // Check if this field is required
        self.check_is_required_string(&msg, "msg")?;
        // Check if there is an joined room
        self.check_is_joined_room()?;

        // Get room (stream) ID and user ID.
        let stream_id = self.room_id.unwrap_or_default();
        let user_id = self.user_id.unwrap_or_default();
        // Prepare data for a new message.
        let create_chat_message = CreateChatMessage::new(stream_id, user_id, &msg);
        // Spawn an async task.
        let addr = ctx.address();
        // Get a clone of the assistant.
        let assistant = self.assistant.clone();
        // Start an additional asynchronous task.
        actix_web::rt::spawn(async move {
            // Check the token for correctness and get the user profile.
            let result = assistant.execute_create_chat_message(create_chat_message).await;

            if let Err(err) = result {
                return addr.do_send(AsyncResultError(err.to_string()));
            }
            let chat_message = result.unwrap();
            let msg = chat_message.msg.unwrap_or_default();
            let date = chat_message.date_update.to_rfc3339_opts(SecondsFormat::Millis, true);
            // Send the "AsyncResultEwsMsg" command for execution.
            addr.do_send(AsyncResultEwsMsg(msg, chat_message.id, date));
        });
        Ok(())
    }

    // // * Send a correction to the message to everyone in the chat. (Server -> Session) *
    // pub fn send_message_to_update(&self, msg_put: &str, date: &str) -> Result<(), String> {    }

    // // * Send a delete message to all chat members. (Server -> Session) *
    // pub fn send_message_to_delete(&self, msg_cut: &str, date: &str) -> Result<(), String> {    }
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
        eprintln!("#!_handle<AsyncResultError>(01) msg.0: {}", &msg.0);
        ctx.text(to_string(&ErrEWS { err: msg.0 }).unwrap());
    }
}

// * * * * Handler for asynchronous response to the "JoinEWS" event * * * *

struct AsyncResultEwsJoin(
    i32,            // room_id
    Option<i32>,    // user_id
    Option<String>, // user_name
);

impl Message for AsyncResultEwsJoin {
    type Result = ();
}

impl Handler<AsyncResultEwsJoin> for ChatWsSession {
    type Result = ();

    fn handle(&mut self, msg: AsyncResultEwsJoin, ctx: &mut Self::Context) {
        // If there is a room name, then leave it.
        if self.room_id.unwrap_or_default() > 0 {
            // Send message about "leave"
            let _ = self.handle_ews_leave(ctx);
        }
        let room_id = msg.0;
        self.user_id = msg.1;
        self.user_name = msg.2;

        // Then send a join message for the new room
        let join_room_srv = JoinRoom(room_id, self.user_name.clone(), ctx.address().recipient());
        // Send the "JoinRoom" command to the server.
        ChatWsServer::from_registry()
            .send(join_room_srv)
            .into_actor(self)
            .then(move |res, act, _ctx| {
                if let Ok(id) = res {
                    act.id = id;
                    act.room_id = Some(room_id);
                }
                fut::ready(())
            })
            .wait(ctx);
    }
}

// * * * * Handler for asynchronous response to the "JoinEWS" event * * * *

struct AsyncResultEwsMsg(
    String, // message
    i32,    // message_id
    String, // message_date
);

impl Message for AsyncResultEwsMsg {
    type Result = ();
}

impl Handler<AsyncResultEwsMsg> for ChatWsSession {
    type Result = ();

    fn handle(&mut self, msg_info: AsyncResultEwsMsg, _ctx: &mut Self::Context) {
        let msg = msg_info.0;
        let id = msg_info.1;
        let member = self.user_name.clone().unwrap_or("".to_owned());
        let date = msg_info.2;
        let msg_str = to_string(&MsgEWS { msg, id, member, date }).unwrap();

        // issue_async comes from having the `BrokerIssue` trait in scope.
        self.issue_system_async(SendMessage(self.room_id.unwrap_or_default(), msg_str));
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
        eprintln!("#!_handler<SaveMessageResult>() msg_res: {:?}", msg_res.0);
        MessageResult(msg_res.0)
    }
}*/
