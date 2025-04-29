use actix::prelude::*;
use actix_broker::BrokerIssue;
// use actix_web::web;
use actix_web_actors::ws;
use chrono::{SecondsFormat, Utc};
use serde_json::to_string;

#[cfg(not(all(test, feature = "mockdata")))]
use crate::chats::chat_message_orm::impls::ChatMessageOrmApp;
#[cfg(all(test, feature = "mockdata"))]
use crate::chats::chat_message_orm::tests::ChatMessageOrmApp;
use crate::chats::chat_models::{
    BlockEWS, CountEWS, EWSType, EchoEWS, ErrEWS, EventWS, MsgCutEWS, MsgEWS, MsgPutEWS, NameEWS, UnblockEWS,
};
use crate::chats::message::{
    BlockClients, BlockSsn, ChatMsgSsn, CommandSrv, CountMembers, JoinRoom, LeaveRoom, SendMessage,
};
use crate::chats::server::WsChatServer;

// ** WsChatSession **

pub struct WsChatSession {
    id: u64,
    room: String,
    name: Option<String>,
    is_blocked: bool,
    chat_message_orm: ChatMessageOrmApp,
}

// ** WsChatSession implementation **

impl WsChatSession {
    pub fn new(
        id: u64,
        room: String,
        name: Option<String>,
        is_blocked: bool,
        chat_message_orm: ChatMessageOrmApp,
    ) -> Self {
        WsChatSession {
            id,
            room,
            name,
            is_blocked,
            chat_message_orm,
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
    // Check if there is a block on sending messages
    fn check_is_blocked(&self) -> Result<(), String> {
        if self.is_blocked {
            Err("There is a block on sending messages.".to_owned())
        } else {
            Ok(())
        }
    }
    // Check if the member has a name or is anonymous
    fn check_is_name(&self) -> Result<(), String> {
        let name = self.name.clone().unwrap_or("".to_owned());
        if name.len() == 0 {
            Err("There was no \"name\" command.".to_owned())
        } else {
            Ok(())
        }
    }
    // Checking before sending a message.
    fn check_before_sending_message(&self) -> Result<(), String> {
        // Check if there is an joined room
        self.check_is_joined_room()?;
        // Check if there is a block on sending messages
        self.check_is_blocked()?;
        // Check if the member has a name or is anonymous
        self.check_is_name()?;
        Ok(())
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

        match event.ews_type() {
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
            EWSType::Count => {
                if let Err(err) = self.count_members(ctx) {
                    ctx.text(to_string(&ErrEWS { err }).unwrap());
                }
            }
            EWSType::Join => {
                let join = event.get("join").unwrap_or("".to_string());
                if let Err(err) = self.join_room(&join, ctx) {
                    ctx.text(to_string(&ErrEWS { err }).unwrap());
                }
            }
            EWSType::Leave => {
                if let Err(err) = self.leave_room(ctx) {
                    ctx.text(to_string(&ErrEWS { err }).unwrap());
                }
            }
            EWSType::Msg => {
                let msg = event.get("msg").unwrap_or("".to_string());
                if let Err(err) = self.send_message(&msg, ctx) {
                    ctx.text(to_string(&ErrEWS { err }).unwrap());
                }
            }
            EWSType::MsgCut => {
                let msg_cut = event.get("msgCut").unwrap_or("".to_string());
                let date = event.get("date").unwrap_or("".to_string());
                // TODO Check the availability of the specified date for the current client.
                if let Err(err) = self.send_message_to_delete(&msg_cut, &date) {
                    ctx.text(to_string(&ErrEWS { err }).unwrap());
                }
            }
            EWSType::MsgPut => {
                let msg_put = event.get("msgPut").unwrap_or("".to_string());
                let date = event.get("date").unwrap_or("".to_string());
                // TODO Check the availability of the specified date for the current client.
                if let Err(err) = self.send_message_to_update(&msg_put, &date) {
                    ctx.text(to_string(&ErrEWS { err }).unwrap());
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
        let client_name = client_name.to_owned();
        // Check if this field is required
        self.check_field_is_required(&client_name, "block")?;
        // Check if there is an joined room
        self.check_is_joined_room()?;

        let room_name = self.room.clone();
        let block_client = BlockClients(room_name, client_name.clone(), is_blocked);

        WsChatServer::from_registry()
            .send(block_client)
            .into_actor(self)
            .then(move |res, _act, ctx| {
                if let Ok(count) = res {
                    #[rustfmt::skip]
                    let str = if is_blocked {
                        to_string(&BlockEWS { block: client_name, count }).unwrap()
                    } else {
                        to_string(&UnblockEWS { unblock: client_name, count }).unwrap()
                    };
                    ctx.text(str);
                }

                fut::ready(())
            })
            .wait(ctx);
        Ok(())
    }

    // ** Count of clients in the room. (Session -> Server) **

    pub fn count_members(&mut self, ctx: &mut ws::WebsocketContext<Self>) -> Result<(), String> {
        // Check if there is an joined room
        self.check_is_joined_room()?;
        let count_members = CountMembers(self.room.clone());

        WsChatServer::from_registry()
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

    // ** Join the client to the chat room. (Session -> Server) **
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
        let join_room_srv = JoinRoom(room_name.clone(), self.name.clone(), ctx.address().recipient());

        WsChatServer::from_registry()
            .send(join_room_srv)
            .into_actor(self)
            .then(|res, act, _ctx| {
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
        let leave_room_srv = LeaveRoom(self.room.clone(), self.id, self.name.clone());
        // issue_sync comes from having the `BrokerIssue` trait in scope.
        self.issue_system_sync(leave_room_srv, ctx);
        // Reset room name.
        self.room = "".to_string();
        Ok(())
    }

    // ** Send a text message to all clients in the room. (Server -> Session) **
    pub fn send_message(&self, msg: &str, _ctx: &mut ws::WebsocketContext<Self>) -> Result<(), String> {
        let msg = msg.to_owned();
        // Check if this field is required
        self.check_field_is_required(&msg, "msg")?;
        // Checking before sending a message.
        self.check_before_sending_message()?;

        let date = Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true);
        let id = date.clone();
        let member = self.name.clone().unwrap_or("".to_owned());
        /*let date = date.to_owned();
        let msg_str = to_string(&MsgEWS { msg, member, date }).unwrap();
        */
        let msg_ews = MsgEWS {
            msg: msg.clone(),
            id,
            member,
            date,
        };

        /*let chat_message_orm = self.chat_message_orm.clone();

        actix_web::rt::spawn(async move {
            let _chat_message_orm2 = chat_message_orm;
            eprintln!("@@ 03 chat_message_orm2 exist");
        });
        eprintln!("@@ 05 chat_message_orm2 exist");
        */

        // let msg_ews = self.prepare_msg_ews(MsgEWS { msg, id, member, date })?;
        // prepare_msg_ews(msg_ews: MsgEWS, chat_message_orm: ChatMessageOrmApp) -> Result<MsgEWS, String>

        // if let Some(chat_message_orm) = self.chat_message_orm.clone() {
        //     // let msg_ews2 = msg_ews.clone();
        //     eprintln!("[01]send_message() let _ = async move "); // #

        //     /*let res = async move {
        //         // wake_and_yield_once().await; // `await` is used within `async` block
        //         // x

        //         let res = prepare_msg_ews(msg_ews2, chat_message_orm).await;

        //         res
        //     };*/
        //     eprintln!("[04]send_message() "); // #
        // }

        let msg_str = to_string(&msg_ews).unwrap();
        // issue_async comes from having the `BrokerIssue` trait in scope.
        self.issue_system_async(SendMessage(self.room.clone(), msg_str));
        Ok(())
    }
    // ** Send a correction to the message to everyone in the chat. (Server -> Session) **
    pub fn send_message_to_update(&self, msg_put: &str, date: &str) -> Result<(), String> {
        let msg_put = msg_put.to_owned();
        // Check if this field is required
        self.check_field_is_required(&msg_put, "msgPut")?;
        // Check if this field is required
        self.check_field_is_required(&date, "date")?;
        // Checking before sending a message.
        self.check_before_sending_message()?;

        let member = self.name.clone().unwrap_or("".to_owned());
        let date = date.to_owned();
        let msg_str = to_string(&MsgPutEWS { msg_put, member, date }).unwrap();

        // issue_async comes from having the `BrokerIssue` trait in scope.
        self.issue_system_async(SendMessage(self.room.clone(), msg_str));
        Ok(())
    }

    // ** Send a delete message to all chat members. (Server -> Session) **
    pub fn send_message_to_delete(&self, msg_cut: &str, date: &str) -> Result<(), String> {
        let msg_cut = msg_cut.to_owned();
        // Check if this field is required
        self.check_field_is_required(&date, "date")?;
        // Checking before sending a message.
        self.check_before_sending_message()?;

        let member = self.name.clone().unwrap_or("".to_owned());
        let date = date.to_owned();
        let msg_str = to_string(&MsgCutEWS { msg_cut, member, date }).unwrap();

        // issue_async comes from having the `BrokerIssue` trait in scope.
        self.issue_system_async(SendMessage(self.room.clone(), msg_str));
        Ok(())
    }
}

// ** **

// Additional data processing (saving in the database).
/*async fn prepare_msg_ews(msg_ews: MsgEWS, chat_message_orm: ChatMessageOrmApp) -> Result<MsgEWS, String> {
    let msg = msg_ews.msg;
    let mut id = "".to_string();
    let member = msg_ews.member;
    let mut date = msg_ews.date;

    let stream_id = 1;
    let user_id = 1;
    let create_chat_message = CreateChatMessage::new(stream_id, user_id, &msg.clone());
    eprintln!("[02a]prepare_msg_ews() web::block(move || ..."); // #
    let opt_chat_message = web::block(move || {
        // Add a new entry (chat_message).
        chat_message_orm
            .create_chat_message(create_chat_message)
            .map_err(|e| {
                log::error!("{}:{}; {}", err::CD_DATABASE, err::MSG_DATABASE, &e);
                format!("{}; {}", err::MSG_DATABASE, &e)
            })
            .ok()
    })
    .await
    .map_err(|e| {
        log::error!("{}:{}; {}", err::CD_BLOCKING, err::MSG_BLOCKING, &e.to_string());
        format!("{}; {}", err::MSG_BLOCKING, &e.to_string())
    })?;
    eprintln!("[02b]prepare_msg_ews() opt_chat_message"); // #
    if let Some(chat_message) = opt_chat_message {
        // msg_ews.msg: String,
        id = chat_message.id.to_string();
        // msg_ews.member: String,
        date = chat_message.date_created.to_rfc3339_opts(SecondsFormat::Millis, true);
    }
    eprintln!("[02c]prepare_msg_ews() MsgEWS ( msg:{msg}, member:{member}, id:{id}, date:{date} )"); // #
    Ok(MsgEWS { msg, member, id, date })
}*/

// ** WsChatSession implementation "Actor" **

impl Actor for WsChatSession {
    type Context = ws::WebsocketContext<Self>;
    // Called when an actor gets polled the first time.
    fn started(&mut self, _ctx: &mut Self::Context) {
        let user = self.name.clone().unwrap_or("".to_owned());
        let user_str = format!("(name: \"{}\", id: {})", user, self.id);
        log::debug!("Session opened for user{} in room \"{}\".", user_str, self.room);
    }
    // Called after an actor is stopped.
    fn stopped(&mut self, _ctx: &mut Self::Context) {
        let user = self.name.clone().unwrap_or("".to_owned());
        let user_str = format!("(name: \"{}\", id: {})", user, self.id);
        log::debug!("Session closed for user{} in room \"{}\".", user_str, self.room);
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
        match command {
            CommandSrv::Block(blocking) => self.handle_block_client(blocking, ctx),
            CommandSrv::Chat(chat_msg) => self.handle_chat_message_ssn(chat_msg, ctx),
        }

        MessageResult(())
    }
}

impl WsChatSession {
    // Handler for "CommandSrv::Block(BlockSsn)".
    fn handle_block_client(&mut self, blocking: BlockSsn, ctx: &mut <Self as actix::Actor>::Context) {
        let BlockSsn(is_blocked) = blocking;
        self.is_blocked = is_blocked;
        let client_name = self.name.clone().unwrap_or("".to_owned());
        #[rustfmt::skip]
        let str = if is_blocked {
            to_string(&BlockEWS { block: client_name, count: 1 }).unwrap()
        } else {
            to_string(&UnblockEWS { unblock: client_name, count: 1 }).unwrap()
        };
        ctx.text(str);
    }

    // Handler for "CommandSrv::Chat(ChatMessageSsn)".
    fn handle_chat_message_ssn(&mut self, chat_msg: ChatMsgSsn, ctx: &mut ws::WebsocketContext<Self>) {
        ctx.text(chat_msg.0);
    }
}
