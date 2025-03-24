use std::collections::HashMap;

use actix::prelude::*;
use actix_broker::BrokerSubscribe;
use serde_json::to_string;

use super::chat_models::{JoinEWS, LeaveEWS};
use super::message::{BlockClients, BlockSsn, ChatMsgSsn, CommandSrv, CountMembers, JoinRoom, LeaveRoom, SendMessage};

type Client = Recipient<CommandSrv>; // ChatMessage

pub struct ClientInfo {
    name: String,
    client: Client,
}

type Room = HashMap<u64, ClientInfo>;

// ** WsChatServer **

#[derive(Default)]
pub struct WsChatServer {
    rooms: HashMap<String, Room>,
}

impl WsChatServer {
    // Take up room for changes.
    fn take_room(&mut self, room_name: &str) -> Option<Room> {
        let room = self.rooms.get_mut(room_name)?;
        let room = std::mem::take(room);
        Some(room)
    }
    // Add a client to the room.
    fn add_client_to_room(&mut self, room_name: &str, id: Option<u64>, client_info: ClientInfo) -> u64 {
        let mut id = id.unwrap_or_else(rand::random);

        if let Some(room) = self.rooms.get_mut(room_name) {
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
        self.rooms.insert(room_name.to_owned(), room);

        id
    }
    // Get the number of clients in the room.
    fn count_clients_in_room(&self, room_name: &str) -> usize {
        self.rooms.get(room_name).map(|room| room.len()).unwrap_or(0)
    }
    // Send a chat message to all members.
    fn send_chat_message_to_clients(&mut self, room_name: &str, msg: &str) -> Option<()> {
        let mut room = self.take_room(room_name)?;

        for (id, client_info) in room.drain() {
            let command_srv = CommandSrv::Chat(ChatMsgSsn(msg.to_owned()));
            if client_info.client.try_send(command_srv).is_ok() {
                self.add_client_to_room(room_name, Some(id), client_info);
            }
        }

        Some(())
    }
}

impl SystemService for WsChatServer {}
impl Supervised for WsChatServer {}

// ** WsChatServer implementation "Actor" **

impl Actor for WsChatServer {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        // Asynchronously subscribe to "SendMessage". (sending `BrokerIssue`.issue_system_async())
        self.subscribe_system_async::<SendMessage>(ctx);
        // Asynchronously subscribe to "LeaveRoomSrv". (sending `BrokerIssue`.issue_system_sync())
        self.subscribe_system_async::<LeaveRoom>(ctx);
    }
}

// ** Blocking clients in a room by name. (Session -> Server) **

impl Handler<BlockClients> for WsChatServer {
    type Result = MessageResult<BlockClients>;

    fn handle(&mut self, msg: BlockClients, _ctx: &mut Self::Context) -> Self::Result {
        let BlockClients(room_name, client_name, is_blocked) = msg.clone();
        if room_name.len() == 0 || client_name.len() == 0 {
            return MessageResult(0);
        }
        let mut count_blocked = 0;
        // Get a chat room by its name.
        if let Some(room) = self.rooms.get_mut(&room_name) {
            // Loop through all chat participants.
            for (_id, client_info) in room {
                // Checking the chat participant name with the name to block.
                if client_info.name.eq(&client_name) {
                    count_blocked += 1;
                    client_info.client.do_send(CommandSrv::Block(BlockSsn(is_blocked)));
                }
            }
        }

        MessageResult(count_blocked)
    }
}

// ** Count of clients in the room. (Session -> Server) **

impl Handler<CountMembers> for WsChatServer {
    type Result = MessageResult<CountMembers>;

    fn handle(&mut self, msg: CountMembers, _ctx: &mut Self::Context) -> Self::Result {
        let CountMembers(room_name) = msg;
        let count = self.count_clients_in_room(&room_name);
        MessageResult(count)
    }
}

// ** Join the client to the chat room. (Session -> Server) **

impl Handler<JoinRoom> for WsChatServer {
    type Result = MessageResult<JoinRoom>;

    fn handle(&mut self, msg: JoinRoom, _ctx: &mut Self::Context) -> Self::Result {
        let JoinRoom(room_name, client_name, client) = msg;
        let name = client_name.unwrap_or("".to_owned());
        let member = name.clone();
        let id = self.add_client_to_room(&room_name, None, ClientInfo { name, client });

        // Get the number of clients in the room.
        let count = self.count_clients_in_room(&room_name);

        let join = room_name.to_owned();
        let join_str = to_string(&JoinEWS { join, member, count }).unwrap();
        // Send a chat message to all members.
        self.send_chat_message_to_clients(&room_name, &join_str);

        MessageResult(id)
    }
}

// ** Leave the client from the chat room. (Session -> Server) **

impl Handler<LeaveRoom> for WsChatServer {
    type Result = ();

    fn handle(&mut self, msg: LeaveRoom, _ctx: &mut Self::Context) {
        let room_name = msg.0;

        if let Some(room) = self.rooms.get_mut(&room_name) {
            // Remove the client from the room.
            let recipient_opt = room.remove(&msg.1);

            let leave = room_name.clone();
            let member = msg.2.unwrap_or("".to_owned());
            // Get the number of clients in the room.
            let count = self.count_clients_in_room(&room_name);
            let leave_str = to_string(&LeaveEWS { leave, member, count }).unwrap();
            // Send a chat message to all members.
            self.send_chat_message_to_clients(&room_name, &leave_str);

            if let Some(client_info) = recipient_opt {
                let command_srv = CommandSrv::Chat(ChatMsgSsn(leave_str.to_owned()));
                client_info.client.do_send(command_srv);
            }
        }
    }
}

// ** Send a text message to all clients in the room. (Server -> Session) **

impl Handler<SendMessage> for WsChatServer {
    type Result = ();

    fn handle(&mut self, msg: SendMessage, _ctx: &mut Self::Context) {
        let SendMessage(room_name, msg) = msg;
        eprintln!("#_hd_SendMessage() room: {}, msg: {}", room_name, msg);
        // Send a chat message to all members.
        self.send_chat_message_to_clients(&room_name, &msg);
    }
}
