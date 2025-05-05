use std::collections::HashMap;

use actix::prelude::*;
use actix_broker::BrokerSubscribe;
use serde_json::to_string;

use crate::chats::chat_event_ws::{JoinEWS, LeaveEWS};
use crate::chats::chat_message::{
    /*BlockClients, BlockSsn,*/ ChatMsgSsn, CommandSrv, CountMembers, JoinRoom, LeaveRoom, SendMessage,
};

type Client = Recipient<CommandSrv>; // ChatMessage

pub struct ClientInfo {
    name: String,
    client: Client,
}

type Room = HashMap<u64, ClientInfo>;

// ** ChatWsServer **

#[derive(Default)]
pub struct ChatWsServer {
    room_map: HashMap<i32, Room>,
}

impl ChatWsServer {
    // Take up room for changes.
    fn take_room(&mut self, room_id: i32) -> Option<Room> {
        let room = self.room_map.get_mut(&room_id)?;
        let room = std::mem::take(room);
        Some(room)
    }
    // Add a client to the room.
    fn add_client_to_room(&mut self, room_id: i32, id: Option<u64>, client_info: ClientInfo) -> u64 {
        let mut id = id.unwrap_or_else(rand::random);

        if let Some(room) = self.room_map.get_mut(&room_id) {
            loop {
                if room.contains_key(&id) {
                    id = rand::random();
                } else {
                    break;
                }
            }
            room.insert(id, client_info);
            return id;
        }

        // Create a new room for the first client
        let mut room: Room = HashMap::new();

        room.insert(id, client_info);
        self.room_map.insert(room_id, room);

        id
    }
    // Get the number of clients in the room.
    fn count_clients_in_room(&self, room_id: i32) -> usize {
        self.room_map.get(&room_id).map(|room| room.len()).unwrap_or(0)
    }
    // Send a chat message to all members.
    fn send_chat_message_to_clients(&mut self, room_id: i32, msg: &str) -> Option<()> {
        let mut room = self.take_room(room_id)?;
        eprintln!("@_send_chat_message() msg: \"{}\"", msg);
        for (id, client_info) in room.drain() {
            let command_srv = CommandSrv::Chat(ChatMsgSsn(msg.to_owned()));
            if client_info.client.try_send(command_srv).is_ok() {
                self.add_client_to_room(room_id, Some(id), client_info);
            }
        }

        Some(())
    }
}

impl SystemService for ChatWsServer {}
impl Supervised for ChatWsServer {}

// ** ChatWsServer implementation "Actor" **

impl Actor for ChatWsServer {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        // Asynchronously subscribe to "SendMessage". (sending `BrokerIssue`.issue_system_async())
        self.subscribe_system_async::<SendMessage>(ctx);
        // Asynchronously subscribe to "LeaveRoom". (sending `BrokerIssue`.issue_system_sync())
        self.subscribe_system_async::<LeaveRoom>(ctx);
    }
}

// ** Blocking clients in a room by name. (Session -> Server) **

// --

// ** Count of clients in the room. (Session -> Server) **

impl Handler<CountMembers> for ChatWsServer {
    type Result = MessageResult<CountMembers>;

    fn handle(&mut self, msg: CountMembers, _ctx: &mut Self::Context) -> Self::Result {
        let CountMembers(room_id) = msg;
        let count = self.count_clients_in_room(room_id);

        MessageResult(count)
    }
}

// ** Join the client to the chat room. (Session -> Server) **

impl Handler<JoinRoom> for ChatWsServer {
    type Result = MessageResult<JoinRoom>;

    fn handle(&mut self, msg: JoinRoom, _ctx: &mut Self::Context) -> Self::Result {
        #[rustfmt::skip]
        eprintln!("@_handler<JoinRoom>() 01 msg.0: {}, msg.1: \"{}\"", msg.0, msg.1.clone().unwrap_or("_".to_string()));
        let JoinRoom(room_id, client_name, client) = msg;
        let name = client_name.unwrap_or("".to_owned());
        let member = name.clone();
        let id = self.add_client_to_room(room_id, None, ClientInfo { name, client });

        // Get the number of clients in the room.
        let count = self.count_clients_in_room(room_id);
        eprintln!("@_handler<JoinRoom>() 02 count: {}", count);
        let join = room_id;
        let join_str = to_string(&JoinEWS { join, member, count }).unwrap();
        // Send a chat message to all members.
        self.send_chat_message_to_clients(room_id, &join_str);

        MessageResult(id)
    }
}

// ** Leave the client from the chat room. (Session -> Server) **

impl Handler<LeaveRoom> for ChatWsServer {
    type Result = ();

    fn handle(&mut self, msg: LeaveRoom, _ctx: &mut Self::Context) {
        let room_id = msg.0;
        let client_id = msg.1;
        let user_name = msg.2;

        if let Some(room) = self.room_map.get_mut(&room_id) {
            // Remove the client from the room.
            let recipient_opt = room.remove(&client_id);
            let leave = room_id;
            let member = user_name.unwrap_or("".to_owned());
            // Get the number of clients in the room.
            let count = self.count_clients_in_room(room_id);
            let leave_str = to_string(&LeaveEWS { leave, member, count }).unwrap();
            // Send a chat message to all members.
            self.send_chat_message_to_clients(room_id, &leave_str);

            if let Some(client_info) = recipient_opt {
                let command_srv = CommandSrv::Chat(ChatMsgSsn(leave_str.to_owned()));
                client_info.client.do_send(command_srv);
            }
        }
    }
}

// ** Send a text message to all clients in the room. (Server -> Session) **

impl Handler<SendMessage> for ChatWsServer {
    type Result = ();

    fn handle(&mut self, msg: SendMessage, _ctx: &mut Self::Context) {
        let SendMessage(room_id, msg_str) = msg;
        // Send a chat message to all members.
        self.send_chat_message_to_clients(room_id, &msg_str);
    }
}

// ** -- **
