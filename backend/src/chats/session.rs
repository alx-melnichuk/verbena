use actix::prelude::*;
use actix_broker::BrokerIssue;
use actix_web_actors::ws;
use serde_json::to_string;

use super::chat_models::{EWSType, EchoEWS, ErrEWS, EventWS, NameEWS};
use super::message::{BlockingClients, BlockingSsn, ChatMessageSsn, CommandSrv, JoinRoomSrv, LeaveRoomSrv};
use super::server::WsChatServer;

// ** WsChatSession **

#[derive(Default)]
pub struct WsChatSession {
    id: u64,
    room: String,
    name: Option<String>,
    is_blocked: bool,
}

// ** WsChatSession implementation **

impl WsChatSession {
    pub fn new(id: u64, room: String, name: Option<String>, is_blocked: bool) -> Self {
        WsChatSession {
            id,
            room,
            name,
            is_blocked,
        }
    }
    // Check if this field is required
    fn check_field_is_required(&self, value: &str, name: &str) -> Result<(), String> {
        if value.len() == 0 {
            Err(format!("The \"{}\" field is required.", name))
        } else {
            Ok(())
        }
    }
    // Check if there is an joined room
    fn check_is_joined_room(&self) -> Result<(), String> {
        if self.room.len() == 0 {
            Err("There was no \"join\" command.".to_owned())
        } else {
            Ok(())
        }
    }

    fn handle_text(&mut self, msg: &str, ctx: &mut ws::WebsocketContext<Self>) {
        // Parse input data of ws event.
        let res_event = EventWS::parsing(msg);
        if let Err(err) = res_event {
            log::debug!("WEBSOCKET: Error: {:?} msg: \"{}\"", err, msg);
            ctx.text(to_string(&ErrEWS { err }).unwrap());
            return;
        }
        let event = res_event.unwrap();

        match event.et {
            EWSType::Echo => {
                let echo = event.get("echo").unwrap_or("".to_string());
                // Check if this field is required
                if let Err(err) = self.check_field_is_required(&echo, "echo") {
                    ctx.text(to_string(&ErrEWS { err }).unwrap());
                } else {
                    ctx.text(to_string(&EchoEWS { echo }).unwrap());
                }
            }
            EWSType::Block => {
                let block = event.get("block").unwrap_or("".to_string());
                if let Err(err) = self.blocking_clients(&block, true, ctx) {
                    ctx.text(to_string(&ErrEWS { err }).unwrap());
                }
            }
            EWSType::Join => {
                let join = event.get("join").unwrap_or("".to_string());
                if let Err(err) = self.join_room(&join, ctx) {
                    ctx.text(to_string(&ErrEWS { err }).unwrap())
                }
            }
            EWSType::Leave => {
                if let Err(err) = self.leave_room(ctx) {
                    ctx.text(to_string(&ErrEWS { err }).unwrap())
                }
            }
            EWSType::Name => {
                let name = event.get("name").unwrap_or("".to_string());
                self.name = if name.len() > 0 { Some(name.clone()) } else { None };
                ctx.text(to_string(&NameEWS { name }).unwrap());
            }
            EWSType::Unblock => {
                let unblock = event.get("unblock").unwrap_or("".to_string());
                if let Err(err) = self.blocking_clients(&unblock, false, ctx) {
                    ctx.text(to_string(&ErrEWS { err }).unwrap());
                }
            }
            _ => {}
        }
    }

    // ** Blocking clients in a room by name. (Session -> Server) **

    pub fn blocking_clients(
        &self,
        client_name: &str,
        is_blocked: bool,
        ctx: &mut ws::WebsocketContext<Self>,
    ) -> Result<(), String> {
        // Check if this field is required
        self.check_field_is_required(&client_name, "block")?;
        // Check if there is an joined room
        self.check_is_joined_room()?;

        let room_name = self.room.clone();
        let block_client = BlockingClients(room_name, client_name.to_owned(), is_blocked);
        eprintln!("@__block() client_name: {}, is_blocked: {}", &client_name, is_blocked); // #
        WsChatServer::from_registry()
            .send(block_client)
            .into_actor(self)
            .then(move |res, _act, _ctx| {
                eprintln!("@__block() res: {:?}", res);
                // if let Ok(id) = res {
                // }

                fut::ready(())
            })
            .wait(ctx);
        /*
        //let block2 = block.to_owned().clone();
        let comm_block_members = CommandSrv::Block(BlockMemberSsn(self.room.clone(), client_name.to_owned().clone()));
        // // issue_sync comes from having the `BrokerIssue` trait in scope.
        let res = self.issue_system_sync(comm_block_members, ctx);
        eprintln!("@__block() issue_system_sync(block_members_msg, ctx): {:?}", res);
        */
        Ok(())
    }

    // ** Join the client to the chat room. **

    pub fn join_room(&mut self, room_name: &str, ctx: &mut ws::WebsocketContext<Self>) -> Result<(), String> {
        // Check if this field is required
        self.check_field_is_required(room_name, "join")?;
        // Check if there was a join to this room.
        if self.room.eq(room_name) {
            return Err(format!("There was already a \"join\" to the \"{}\" room.", room_name));
        }
        // If there is a room name, then leave it.
        if self.room.len() > 0 {
            // Send message about "leave"
            let _ = self.leave_room(ctx);
        }
        let room_name = room_name.to_owned();
        // Then send a join message for the new room
        let join_room_srv = JoinRoomSrv(room_name.clone(), self.name.clone(), ctx.address().recipient());

        WsChatServer::from_registry()
            .send(join_room_srv)
            .into_actor(self)
            .then(move |res, act, _ctx| {
                if let Ok(id) = res {
                    act.id = id;
                    act.room = room_name;
                }

                fut::ready(())
            })
            .wait(ctx);
        Ok(())
    }

    // ** Leave the client from the chat room. (Session -> Server) **
    pub fn leave_room(&mut self, ctx: &mut ws::WebsocketContext<Self>) -> Result<(), String> {
        // Check if there is an joined room
        self.check_is_joined_room()?;
        // Send a message about leaving the room.
        let leave_room_srv = LeaveRoomSrv(self.room.clone(), self.id, self.name.clone());
        // issue_sync comes from having the `BrokerIssue` trait in scope.
        self.issue_system_sync(leave_room_srv, ctx);
        // Reset room name.
        self.room = "".to_string();
        Ok(())
    }
}

// ** WsChatSession implementation "Actor" **

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

// ** WsChatSession implementation "StreamHandler<Result<ws::Message, ws::ProtocolError>>" **

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
                let _ = self.leave_room(ctx);
                ctx.close(reason);
                ctx.stop();
            }
            _ => {}
        }
    }
}

// ** WsChatSession implementation "Handler<CommandSrv>" **

impl Handler<CommandSrv> for WsChatSession {
    type Result = MessageResult<CommandSrv>;

    fn handle(&mut self, command: CommandSrv, ctx: &mut Self::Context) -> Self::Result {
        eprintln!("@_hd_Ssn_CmndSrv() msg: {:?}", command);
        //ctx.text(msg.0);
        match command {
            CommandSrv::Blocking(blocking) => self.handle_block_client(blocking, ctx),
            CommandSrv::Chat(chat_msg) => self.handle_chat_message_ssn(chat_msg, ctx),
        }

        MessageResult(())
    }
}

impl WsChatSession {
    // Handler for "CommandSrv::Block(BlockSsn)".
    fn handle_block_client(&mut self, blocking: BlockingSsn, _ctx: &mut <Self as actix::Actor>::Context) {
        let BlockingSsn(is_blocked) = blocking;
        #[rustfmt::skip]
        eprintln!("@_handle_block_client() id: {}, room: {}, name: {:?}, is_blocked: {}", self.id, self.room, self.name, is_blocked);
        self.is_blocked = is_blocked;
    }

    // Handler for "CommandSrv::Chat(ChatMessageSsn)".
    fn handle_chat_message_ssn(&mut self, chat_msg: ChatMessageSsn, ctx: &mut ws::WebsocketContext<Self>) {
        ctx.text(chat_msg.0);
    }
}
