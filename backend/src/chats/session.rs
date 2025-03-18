use actix::prelude::*;
use actix_broker::BrokerIssue;
use actix_web_actors::ws;

use super::chat_models::{WSEvent, WSEventType};
use super::message::{ChatMessage, CountMembers, JoinRoom, LeaveRoom, SendMessage};
use super::server::WsChatServer;

pub const UNDEFINED_ROOM_NAME: &str = "Undefined room name.";

#[derive(Default)]
pub struct WsChatSession {
    id: u64,
    room: String,
    name: Option<String>,
}

impl WsChatSession {
    pub fn new(id: u64, room: String, name: Option<String>) -> Self {
        WsChatSession { id, room, name }
    }

    // ** Count of clients in the room. **

    pub fn count_members(&mut self, ctx: &mut ws::WebsocketContext<Self>) {
        // Check if there is an joined room
        if let Err(err) = self.check_is_joined_room() {
            return ctx.text(err);
        }
        let count_members_msg = CountMembers(self.room.clone());

        WsChatServer::from_registry()
            .send(count_members_msg)
            .into_actor(self)
            .then(|res, _act, ctx| {
                if let Ok(count) = res {
                    ctx.text(WSEvent::count(count));
                }

                fut::ready(())
            })
            .wait(ctx);
    }

    // ** Block the client in the room. **

    pub fn block(&self, block: &str, ctx: &mut ws::WebsocketContext<Self>) {
        // Check if this field is required
        if let Err(err) = self.check_field_is_required(&block, "block") {
            return ctx.text(err);
        }
        // Check if there is an joined room
        if let Err(err) = self.check_is_joined_room() {
            return ctx.text(err);
        }
        // eprintln!("@__block() block: {}", block);
        // let block_msg = Block(self.room.clone(), block.to_owned());
        // // issue_async comes from having the `BrokerIssue` trait in scope.
        // self.issue_system_async(block_msg);
    }

    // ** Join the client to the chat room. **

    pub fn join_room(&mut self, room_name: &str, ctx: &mut ws::WebsocketContext<Self>) {
        // Check if this field is required
        if let Err(err) = self.check_field_is_required(room_name, "join") {
            return ctx.text(err);
        }
        // Check if there was a join to this room.
        if self.room.eq(room_name) {
            let err = format!("There was already a \"join\" to the \"{}\" room.", room_name);
            return ctx.text(WSEvent::err(err));
        }

        if self.room.len() > 0 {
            // First send a leave message for the current room
            let leave_msg = LeaveRoom(self.room.clone(), self.id, self.name.clone());
            // issue_sync comes from having the `BrokerIssue` trait in scope.
            self.issue_system_sync(leave_msg, ctx);
        }
        let room_name = room_name.to_owned();
        // Then send a join message for the new room
        let join_room_msg = JoinRoom(room_name.clone(), self.name.clone(), ctx.address().recipient());

        WsChatServer::from_registry()
            .send(join_room_msg)
            .into_actor(self)
            .then(move |res, act, _ctx| {
                if let Ok(id) = res {
                    act.id = id;
                    act.room = room_name;
                }

                fut::ready(())
            })
            .wait(ctx);
    }

    // ** Leave the client from the chat room. **

    pub fn leave_room(&mut self, ctx: &mut ws::WebsocketContext<Self>) {
        // Check if there is an joined room
        if let Err(err) = self.check_is_joined_room() {
            return ctx.text(err);
        }
        // Send a message about leaving the current room.
        let leave_room_msg = LeaveRoom(self.room.clone(), self.id, self.name.clone());
        // issue_sync comes from having the `BrokerIssue` trait in scope.
        self.issue_system_sync(leave_room_msg, ctx);
        // Reset room name.
        self.room = "".to_string();
    }

    // ** Send a message to everyone in the chat room. **

    pub fn send_message(&self, msg: &str, date: &str, ctx: &mut ws::WebsocketContext<Self>) {
        // Check if this field is required
        if let Err(err) = self.check_field_is_required(&msg, "msg") {
            return ctx.text(err);
        }
        // Check if there is an joined room
        if let Err(err) = self.check_is_joined_room() {
            return ctx.text(err);
        }
        let member = self.name.clone().unwrap_or("".to_owned());
        let msg_str = WSEvent::msg(msg.to_owned(), member, date.to_owned());
        let send_message = SendMessage(self.room.clone(), self.id, msg_str);

        // issue_async comes from having the `BrokerIssue` trait in scope.
        self.issue_system_async(send_message);
    }

    // ** Send a delete message to all chat members. **
    pub fn send_message_cut(&self, msg_cut: &str, date: &str, ctx: &mut ws::WebsocketContext<Self>) {
        // Check if there is an joined room
        if let Err(err) = self.check_is_joined_room() {
            return ctx.text(err);
        }
        let member = self.name.clone().unwrap_or("".to_owned());
        let msg_cut_str = WSEvent::msg_cut(msg_cut.to_owned(), member, date.to_owned());
        let send_message = SendMessage(self.room.clone(), self.id, msg_cut_str);

        // issue_async comes from having the `BrokerIssue` trait in scope.
        self.issue_system_async(send_message);
    }

    // ** Send a correction to the message to everyone in the chat. **

    pub fn send_message_put(&self, msg_put: &str, date: &str, ctx: &mut ws::WebsocketContext<Self>) {
        // Check if this field is required
        if let Err(err) = self.check_field_is_required(&msg_put, "msg_put") {
            return ctx.text(err);
        }
        // Check if there is an joined room
        if let Err(err) = self.check_is_joined_room() {
            return ctx.text(err);
        }
        let member = self.name.clone().unwrap_or("".to_owned());
        let msg_put_str = WSEvent::msg_put(msg_put.to_owned(), member, date.to_owned());
        let send_message = SendMessage(self.room.clone(), self.id, msg_put_str);

        // issue_async comes from having the `BrokerIssue` trait in scope.
        self.issue_system_async(send_message);
    }

    // Check if there is an joined room
    fn check_is_joined_room(&self) -> Result<(), String> {
        if self.room.len() == 0 {
            Err(WSEvent::err("There was no \"join\" command.".to_owned()))
        } else {
            Ok(())
        }
    }

    // Check if this field is required
    fn check_field_is_required(&self, value: &str, name: &str) -> Result<(), String> {
        if value.len() == 0 {
            Err(WSEvent::err(format!("The \"{}\" field is required.", name)))
        } else {
            Ok(())
        }
    }

    fn handle_text(&mut self, msg: &str, ctx: &mut ws::WebsocketContext<Self>) {
        // Parse input data of ws event.
        let res_event = WSEvent::parsing(msg);
        if let Err(err) = res_event {
            log::debug!("WEBSOCKET: Error: {:?} msg: \"{}\"", err, msg);
            ctx.text(WSEvent::err(err));
            return;
        }
        let event = res_event.unwrap();

        match event.et {
            WSEventType::Count => {
                self.count_members(ctx);
            }
            WSEventType::Echo => {
                let echo = event.get("echo").unwrap_or("".to_string());
                // Check if this field is required
                if let Err(err) = self.check_field_is_required(&echo, "echo") {
                    ctx.text(err)
                } else {
                    ctx.text(WSEvent::echo(echo));
                }
            }
            WSEventType::Name => {
                let name = event.get("name").unwrap_or("".to_string());
                self.name = if name.len() > 0 { Some(name.clone()) } else { None };
                ctx.text(WSEvent::name(name));
            }
            WSEventType::Block => {
                let block = event.get("block").unwrap_or("".to_string());
                // eprintln!("@__handle_text() block: {}", block);
                self.block(&block, ctx);
            }
            WSEventType::Join => {
                let join = event.get("join").unwrap_or("".to_string());
                self.join_room(&join, ctx);
            }
            WSEventType::Leave => {
                self.leave_room(ctx);
            }
            WSEventType::Msg => {
                let msg = event.get("msg").unwrap_or("".to_string());
                self.send_message(&msg, "", ctx);
            }
            WSEventType::MsgCut => {
                let msg_cut = event.get("msg_cut").unwrap_or("".to_string());
                self.send_message_cut(&msg_cut, "", ctx);
            }
            WSEventType::MsgPut => {
                let msg_put = event.get("msg_put").unwrap_or("".to_string());
                self.send_message_put(&msg_put, "", ctx);
            }
            _ => {}
        }
    }
}

impl Actor for WsChatSession {
    type Context = ws::WebsocketContext<Self>;
    // Called when an actor gets polled the first time.
    fn started(&mut self, _ctx: &mut Self::Context) {
        let user = self.name.clone().unwrap_or("".to_owned());
        log::debug!("Session opened for {} ({}) in room \"{}\".", user, self.id, self.room);
    }
    // Called after an actor is stopped.
    fn stopped(&mut self, _ctx: &mut Self::Context) {
        let user = self.name.clone().unwrap_or("".to_owned());
        log::debug!("Session closed for {}({}) in room \"{}\".", user, self.id, self.room);
    }
}

impl Handler<ChatMessage> for WsChatSession {
    type Result = ();

    fn handle(&mut self, msg: ChatMessage, ctx: &mut Self::Context) {
        ctx.text(msg.0);
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsChatSession {
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
                // Send message about "leave"
                self.leave_room(ctx);
                ctx.close(reason);
                ctx.stop();
            }
            _ => {}
        }
    }
}
