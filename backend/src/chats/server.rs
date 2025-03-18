use std::collections::HashMap;

use actix::prelude::*;
use actix_broker::BrokerSubscribe;

use super::{
    chat_models::WSEvent,
    message::{ChatMessage, CountMembers, JoinRoom, LeaveRoom, SendMessage},
};

type Client = Recipient<ChatMessage>;

pub struct ClientInfo {
    name: String,
    client: Client,
}

type Room = HashMap<u64, ClientInfo>;

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
    fn add_client_to_room(&mut self, room_name: &str, id: Option<u64>, name: String, client: Client) -> u64 {
        let mut id = id.unwrap_or_else(rand::random);

        if let Some(room) = self.rooms.get_mut(room_name) {
            loop {
                if room.contains_key(&id) {
                    id = rand::random();
                } else {
                    break;
                }
            }
            room.insert(id, ClientInfo { name, client });
            return id;
        }

        // Create a new room for the first client
        let mut room: Room = HashMap::new();

        room.insert(id, ClientInfo { name, client });
        self.rooms.insert(room_name.to_owned(), room);

        id
    }
    // Get the number of clients in the room.
    fn count_clients_in_room(&self, room_name: &str) -> usize {
        self.rooms.get(room_name).map(|room| room.len()).unwrap_or(0)
    }
    // Send a message to all clients of the room.
    fn send_message_to_chat(&mut self, room_name: &str, msg: &str) -> Option<()> {
        let mut room = self.take_room(room_name)?;

        for (id, client_info) in room.drain() {
            if client_info.client.try_send(ChatMessage(msg.to_owned())).is_ok() {
                self.add_client_to_room(room_name, Some(id), client_info.name, client_info.client);
            }
        }

        Some(())
    }
}

impl Actor for WsChatServer {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.subscribe_system_async::<LeaveRoom>(ctx);
        self.subscribe_system_async::<SendMessage>(ctx);
    }
}

// ** Count of clients in the room. **

impl Handler<CountMembers> for WsChatServer {
    type Result = MessageResult<CountMembers>;

    fn handle(&mut self, msg: CountMembers, _ctx: &mut Self::Context) -> Self::Result {
        let CountMembers(room_name) = msg;
        let count = self.count_clients_in_room(&room_name);
        MessageResult(count)
    }
}

// ** Join the client to the chat room. **

impl Handler<JoinRoom> for WsChatServer {
    type Result = MessageResult<JoinRoom>;

    fn handle(&mut self, msg: JoinRoom, _ctx: &mut Self::Context) -> Self::Result {
        let JoinRoom(room_name, client_name, client) = msg;
        let member = client_name.unwrap_or("".to_owned());
        let id = self.add_client_to_room(&room_name, None, member.clone(), client);

        let count = self.count_clients_in_room(&room_name);

        let join_str = WSEvent::join(room_name.clone(), member, count);
        self.send_message_to_chat(&room_name, &join_str);

        MessageResult(id)
    }
}

// ** Leave the client from the chat room. **

impl Handler<LeaveRoom> for WsChatServer {
    type Result = ();

    fn handle(&mut self, msg: LeaveRoom, _ctx: &mut Self::Context) {
        let room_name = msg.0;
        //self.find_client_in_room("User1", &room_name);

        if let Some(room) = self.rooms.get_mut(&room_name) {
            let recipient_opt = room.remove(&msg.1);

            let count = self.count_clients_in_room(&room_name);
            let member = msg.2.unwrap_or("".to_owned());
            let leave_str = WSEvent::leave(room_name.clone(), member, count);

            self.send_message_to_chat(&room_name, &leave_str);

            if let Some(client_info) = recipient_opt {
                client_info.client.do_send(ChatMessage(leave_str.to_owned()));
            }
        }
    }
}

// ** Send a message to everyone in the chat room. **

impl Handler<SendMessage> for WsChatServer {
    type Result = ();

    fn handle(&mut self, msg: SendMessage, _ctx: &mut Self::Context) {
        let SendMessage(room_name, _id, msg) = msg;
        self.send_message_to_chat(&room_name, &msg);
    }
}

impl SystemService for WsChatServer {}
impl Supervised for WsChatServer {}
