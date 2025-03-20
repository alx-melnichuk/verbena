use std::collections::HashMap;

use actix::prelude::*;
use actix_broker::BrokerSubscribe;
use serde_json::to_string;

use super::chat_models::{JoinEWS, LeaveEWS};
use super::message::{BlockingClients, BlockingSsn, ChatMessageSsn, CommandSrv, JoinRoomSrv, LeaveRoomSrv};

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
            let command_srv = CommandSrv::Chat(ChatMessageSsn(msg.to_owned()));
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
        self.subscribe_system_async::<BlockingClients>(ctx);
        // self.subscribe_system_async::<CommandSrv>(ctx);
        self.subscribe_system_async::<LeaveRoomSrv>(ctx);
    }
}

// ** WsChatServer implementation "Handler<BlockClient>" **

impl Handler<BlockingClients> for WsChatServer {
    type Result = MessageResult<BlockingClients>;

    fn handle(&mut self, msg: BlockingClients, _ctx: &mut Self::Context) -> Self::Result {
        let BlockingClients(room_name, client_name, is_blocked) = msg.clone();
        eprintln!(
            "#_hd_BlockClient() room: {}, client: {}, is_blocked: {}",
            room_name, client_name, is_blocked
        );
        if room_name.len() == 0 || client_name.len() == 0 {
            return MessageResult(());
        }
        let mut count_blocked = 0;
        // Get a chat room by its name.
        if let Some(room) = self.rooms.get_mut(&room_name) {
            // Loop through all chat participants.
            for (id, client_info) in room {
                let r1 = client_info.name.eq(&client_name);
                #[rustfmt::skip]
                eprintln!("#_BlockMembers() id: {id}, cl_name: {}, cl_name.eq(name): {}", client_info.name, r1);
                // Checking the chat participant name with the name to block.
                if client_info.name.eq(&client_name) {
                    count_blocked += 1;
                    eprintln!(
                        "#_BlockMembers client.do_send(CommandSrv::Block(BlockSsn({})));",
                        is_blocked
                    );
                    client_info.client.do_send(CommandSrv::Blocking(BlockingSsn(is_blocked)));
                }
            }
        }
        eprintln!("#_BlockMembers() count_blocked: {}", count_blocked);
        MessageResult(())
    }
}

// ** Join the client to the chat room. (Session -> Server) **

impl Handler<JoinRoomSrv> for WsChatServer {
    type Result = MessageResult<JoinRoomSrv>;

    fn handle(&mut self, msg: JoinRoomSrv, _ctx: &mut Self::Context) -> Self::Result {
        let JoinRoomSrv(room_name, client_name, client) = msg;
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

impl Handler<LeaveRoomSrv> for WsChatServer {
    type Result = ();

    fn handle(&mut self, msg: LeaveRoomSrv, _ctx: &mut Self::Context) {
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
                let command_srv = CommandSrv::Chat(ChatMessageSsn(leave_str.to_owned()));
                client_info.client.do_send(command_srv);
            }
        }
    }
}
