use actix::prelude::*;
// use actix_broker::BrokerIssue;
// use actix_web::web;
use actix_web_actors::ws;
// use chrono::{SecondsFormat, Utc};
use serde_json::to_string;

use crate::chats::chat_event_ws::{
    /*BlockEWS, CountEWS,*/ EWSType, /*EchoEWS,*/ ErrEWS, EventWS,
    /*MsgCutEWS, MsgEWS, MsgPutEWS,*/ NameEWS, /*UnblockEWS,*/
};
use crate::chats::chat_message::{
    /*BlockClients, BlockSsn, ChatMsgSsn,*/
    CommandSrv, /*CountMembers,*/ JoinRoom, /*LeaveRoom, SaveMessageResult, SendMessage,*/
};
// use crate::chats::chat_message_models::CreateChatMessage;
#[cfg(not(all(test, feature = "mockdata")))]
use crate::chats::chat_message_orm::impls::ChatMessageOrmApp;
#[cfg(all(test, feature = "mockdata"))]
use crate::chats::chat_message_orm::tests::ChatMessageOrmApp;
// use crate::chats::chat_message_storage::execute_create_chat_message;
use crate::chats::chat_ws_server::ChatWsServer;

pub const PARAMETER_NOT_DEFINED: &str = "parameter not defined";
pub const THERE_WAS_ALREADY_JOIN_TO_ROOM: &str = "There was already a \"join\" to the room";

// ** ChatWsSession **

pub struct ChatWsSession {
    id: u64,
    room_id: Option<i32>,
    user_id: Option<i32>,
    user_name: Option<String>,
    is_blocked: bool,
    chat_message_orm: ChatMessageOrmApp,
}

// ** ChatWsSession implementation **

impl ChatWsSession {
    pub fn new(
        id: u64,
        room_id: Option<i32>,
        user_id: Option<i32>,
        user_name: Option<String>,
        is_blocked: bool,
        chat_message_orm: ChatMessageOrmApp,
    ) -> Self {
        let name = user_name.clone().unwrap_or("@".to_string());
        eprintln!("}}ChatWsSession() user_name: {}", name);
        ChatWsSession {
            id,
            room_id,
            user_id,
            user_name,
            is_blocked,
            chat_message_orm,
        }
    }
    // Check if this field is required
    // #[rustfmt::skip]
    // fn check_is_required_string(&self, value: &str, name: &str) -> Result<(), String> {
    //     if value.len() == 0 { Err(format!("\"{}\" {}", name, PARAMETER_NOT_DEFINED)) } else { Ok(()) }
    // }
    #[rustfmt::skip]
    fn check_is_required_i32(&self, value: i32, name: &str) -> Result<(), String> {
        if value < 0 { Err(format!("\"{}\" {}", name, PARAMETER_NOT_DEFINED)) } else { Ok(()) }
    }
    // // Check if there is an joined room
    // fn check_is_joined_room(&self) -> Result<(), String> {
    //     if self.room.len() == 0 { Err("There was no \"join\" command.".to_owned()) } else { Ok(()) }
    // }
    // // Check if there is a block on sending messages
    // fn check_is_blocked(&self) -> Result<(), String> {
    //     if self.is_blocked { Err("There is a block on sending messages.".to_owned()) } else { Ok(()) }
    // }
    // // Check if the member has a name or is anonymous
    // fn check_is_name(&self) -> Result<(), String> {
    //     let name = self.name.clone().unwrap_or("".to_owned());
    //     if name.len() == 0 { Err("There was no \"name\" command.".to_owned()) } else { Ok(()) }
    // }

    fn handle_text(&mut self, msg: &str, ctx: &mut ws::WebsocketContext<Self>) {
        eprintln!("}}_handle_text() msg: {}", msg);
        // Parse input data of ws event.
        let res_event = EventWS::parsing(msg);
        if let Err(err) = res_event {
            log::debug!("WEBSOCKET: Error: {:?} msg: \"{}\"", err, msg);
            ctx.text(to_string(&ErrEWS { err }).unwrap());
            return;
        }
        let event = res_event.unwrap();

        match event.ews_type() {
            EWSType::Join => {
                let join = event.get_i32("join").unwrap_or(-1);
                if let Err(err) = self.do_join_room(join, ctx) {
                    ctx.text(to_string(&ErrEWS { err }).unwrap());
                }
            }
            EWSType::Name => {
                let name2 = event.get_string("name").unwrap_or("".to_string());
                // For an authorized user, id and name are defined.
                let id = self.user_id.clone().unwrap_or_default();
                let name = self.user_name.clone().unwrap_or(name2);
                ctx.text(to_string(&NameEWS { id, name }).unwrap());
            }
            _ => {}
        }
    }

    // // ** Blocking clients in a room by name. (Session -> Server) **
    // pub fn blocking_clients(&self,client_name: &str,is_blocked: bool,ctx: &mut ws::WebsocketContext<Self>,) -> Result<(), String> {    }

    // // ** Count of clients in the room. (Session -> Server) **
    // pub fn count_members(&mut self, ctx: &mut ws::WebsocketContext<Self>) -> Result<(), String> {    }

    // ** Join the client to the chat room. (Session -> Server) **
    pub fn do_join_room(&mut self, room_id: i32, ctx: &mut ws::WebsocketContext<Self>) -> Result<(), String> {
        eprintln!("}}_do_join_room() room_id: {}", room_id);
        // Check if this field is required
        self.check_is_required_i32(room_id, "join")?;
        let curr_room_id = self.room_id.unwrap_or(-1);
        // Check if there was a join to this room.
        if curr_room_id == room_id {
            return Err(format!("{} {}.", THERE_WAS_ALREADY_JOIN_TO_ROOM, room_id));
        }
        // If there is a room name, then leave it.
        // if curr_room_id > 0 {
        //     // Send message about "leave"
        //     let _ = self.do_leave_room(ctx);
        // }
        #[rustfmt::skip]
        eprintln!("}}_do_join_room() user_name: {}", self.user_name.clone().unwrap_or("^".to_string()));
        // Then send a join message for the new room
        let join_room_srv = JoinRoom(room_id, self.user_name.clone(), ctx.address().recipient());

        ChatWsServer::from_registry()
            .send(join_room_srv)
            .into_actor(self)
            .then(move |res, act, _ctx| {
                let mut id2 = 0;
                if let Ok(id) = res {
                    act.id = id;
                    act.room_id = Some(room_id);
                    id2 = id;
                }
                eprintln!("}}_do_join_room(2) room_id: {}, id: {}", room_id, id2);
                fut::ready(())
            })
            .wait(ctx);
        Ok(())
    }

    // // ** Leave the client from the chat room. (Session -> Server) **
    // pub fn leave_room(&mut self, ctx: &mut ws::WebsocketContext<Self>) -> Result<(), String> {    }

    // // ** Send a text message to all clients in the room. (Server -> Session) **
    // pub fn send_message(&self, msg: &str, ctx: &mut ws::WebsocketContext<Self>) -> Result<(), String> {    }

    // // ** Send a correction to the message to everyone in the chat. (Server -> Session) **
    // pub fn send_message_to_update(&self, msg_put: &str, date: &str) -> Result<(), String> {    }

    // // ** Send a delete message to all chat members. (Server -> Session) **
    // pub fn send_message_to_delete(&self, msg_cut: &str, date: &str) -> Result<(), String> {    }
}

// ** **

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
                self.handle_text(text.trim(), ctx);
            }
            ws::Message::Close(reason) => {
                // // Send message about "leave"
                // let _ = self.leave_room(ctx);
                ctx.close(reason);
                ctx.stop();
            }
            _ => {}
        }
    }
}

// ** ChatWsSession implementation "Handler<CommandSrv>" **

impl Handler<CommandSrv> for ChatWsSession {
    type Result = MessageResult<CommandSrv>;

    fn handle(&mut self, _command: CommandSrv, _ctx: &mut Self::Context) -> Self::Result {
        // match command {
        //     CommandSrv::Block(blocking) => self.handle_block_client(blocking, ctx),
        //     CommandSrv::Chat(chat_msg) => self.handle_chat_message_ssn(chat_msg, ctx),
        // }

        MessageResult(())
    }
}

/*impl ChatWsSession {
    // Handler for "CommandSrv::Block(BlockSsn)".
    fn handle_block_client(&mut self, blocking: BlockSsn, ctx: &mut <Self as actix::Actor>::Context) {
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
    }

    // Handler for "CommandSrv::Chat(ChatMessageSsn)".
    fn handle_chat_message_ssn(&mut self, chat_msg: ChatMsgSsn, ctx: &mut ws::WebsocketContext<Self>) {
        ctx.text(chat_msg.0);
    }
}*/

// ** ChatWsSession implementation "Handler<SaveMessageResult>" **

/*impl Handler<SaveMessageResult> for ChatWsSession {
    type Result = MessageResult<SaveMessageResult>;

    fn handle(&mut self, msg_res: SaveMessageResult, _ctx: &mut Self::Context) -> Self::Result {
        eprintln!("handler<SaveMessageResult>() msg_res: {:?}", msg_res.0);
        MessageResult(msg_res.0)
    }
}*/
