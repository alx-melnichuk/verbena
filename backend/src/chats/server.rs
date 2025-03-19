use std::collections::HashMap;

use actix::prelude::*;
use actix_broker::BrokerSubscribe;

use super::{
    chat_models::WSEvent,
    message::{BlockMembers, ChatMessage, CountMembers, JoinRoom, SrvCommand, SrvLeaveRoom, SrvSendMessage},
};

type Client = Recipient<ChatMessage>;
type ClientSession = Recipient<SrvCommand>; // BlockMembers, ChatMessage

pub struct ClientInfo {
    name: String,
    client: Client,
    client_session: ClientSession,
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
    fn add_client_to_room(
        &mut self,
        room_name: &str,
        id: Option<u64>,
        name: String,
        client: Client,
        client_session: ClientSession,
    ) -> u64 {
        let mut id = id.unwrap_or_else(rand::random);

        if let Some(room) = self.rooms.get_mut(room_name) {
            loop {
                if room.contains_key(&id) {
                    id = rand::random();
                } else {
                    break;
                }
            }
            room.insert(
                id,
                ClientInfo {
                    name,
                    client,
                    client_session,
                },
            );
            return id;
        }

        // Create a new room for the first client
        let mut room: Room = HashMap::new();

        room.insert(
            id,
            ClientInfo {
                name,
                client,
                client_session,
            },
        );
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
                self.add_client_to_room(
                    room_name,
                    Some(id),
                    client_info.name,
                    client_info.client,
                    client_info.client_session,
                );
            }
        }

        Some(())
    }
}

// ** WsChatServer implementation "Actor" **

impl Actor for WsChatServer {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.subscribe_system_async::<SrvCommand>(ctx);
        self.subscribe_system_async::<SrvLeaveRoom>(ctx);
        self.subscribe_system_async::<SrvSendMessage>(ctx);
    }
}

// ** WsChatServer implementation "Handler<SrvCommand>" **

impl Handler<SrvCommand> for WsChatServer {
    type Result = MessageResult<SrvCommand>;

    fn handle(&mut self, msg: SrvCommand, ctx: &mut Self::Context) -> Self::Result {
        match msg {
            SrvCommand::Block(block_members) => self.handle_block_members(block_members, ctx),
            SrvCommand::Chat(_chat_message) => eprintln!("SrvCommand::Chat"),
        }
        MessageResult(())
    }
}

// ** WsChatServer implementation of "SrvCommand" command processing. **

impl WsChatServer {
    // ** Block clients in a room by name. **
    fn handle_block_members(&mut self, msg: BlockMembers, _ctx: &mut <Self as actix::Actor>::Context) {
        let BlockMembers(room_name, client_name) = msg.clone();
        eprintln!("#_BlockMembers() room: \"{}\", client: \"{}\"", room_name, client_name);
        if room_name.len() == 0 || client_name.len() == 0 {
            return;
        }

        if let Some(room) = self.rooms.get_mut(&room_name) {
            for (id, client_info) in room {
                let r1 = client_info.name.eq(&client_name);
                #[rustfmt::skip]
                eprintln!("#_BlockMembers() id: {id}, cl_name: {}, cl_name.eq(name): {}", client_info.name, r1);

                if client_info.name.eq(&client_name) {
                    let block_str = WSEvent::block(client_info.name.clone(), None);
                    eprintln!("#_BlockMembers client.do_send({})", block_str);
                    client_info.client_session.do_send(SrvCommand::Block(msg.clone()));
                }
            }
            // let count = self.count_clients_in_room(&room_name);
            // let member = msg.2.unwrap_or("".to_owned());
            // let leave_str = WSEvent::leave(room_name.clone(), member, count);
            // self.send_message_to_chat(&room_name, &leave_str);
        }
    }

    // ** Send a chat message to all clients in the room. **
    fn handle_chat_message(&mut self, msg: ChatMessage, _ctx: &mut <Self as actix::Actor>::Context) {
        eprintln!("handle_chat_message");
    }
}

// ** ?? Count of clients in the room. **

impl Handler<CountMembers> for WsChatServer {
    type Result = MessageResult<CountMembers>;

    fn handle(&mut self, msg: CountMembers, _ctx: &mut Self::Context) -> Self::Result {
        let CountMembers(room_name) = msg;
        let count = self.count_clients_in_room(&room_name);
        MessageResult(count)
    }
}

// ** ?? Join the client to the chat room. **

impl Handler<JoinRoom> for WsChatServer {
    type Result = MessageResult<JoinRoom>;

    fn handle(&mut self, msg: JoinRoom, _ctx: &mut Self::Context) -> Self::Result {
        let JoinRoom(room_name, client_name, client, client_session) = msg;
        let member = client_name.unwrap_or("".to_owned());
        let id = self.add_client_to_room(&room_name, None, member.clone(), client, client_session);

        let count = self.count_clients_in_room(&room_name);

        let join_str = WSEvent::join(room_name.clone(), member, count);
        self.send_message_to_chat(&room_name, &join_str);

        MessageResult(id)
    }
}

// ** Leave the client from the chat room. (Session -> Server) **

impl Handler<SrvLeaveRoom> for WsChatServer {
    type Result = ();

    fn handle(&mut self, msg: SrvLeaveRoom, _ctx: &mut Self::Context) {
        let room_name = msg.0;

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

// ** Send a message to everyone in the chat room. (Session -> Server) **

impl Handler<SrvSendMessage> for WsChatServer {
    type Result = ();

    fn handle(&mut self, msg: SrvSendMessage, _ctx: &mut Self::Context) {
        let SrvSendMessage(room_name, _id, msg) = msg;
        self.send_message_to_chat(&room_name, &msg);
    }
}

impl SystemService for WsChatServer {}
impl Supervised for WsChatServer {}
