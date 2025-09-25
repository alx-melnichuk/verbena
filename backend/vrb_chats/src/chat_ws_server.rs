use std::collections::HashMap;

use actix::prelude::*;
use actix_broker::BrokerSubscribe;
use log::debug;
use rand;
use serde_json::to_string;

use crate::{
    chat_event_ws::{JoinEWS, LeaveEWS},
    chat_message::{BlockClient, BlockSsn, ChatMsgSsn, CommandSrv, CountMembers, JoinRoom, LeaveRoom, SendMessage},
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
    /** Add a client to the room. ("count" - number of members, "id" - new member ID) */
    fn add_client_to_room(&mut self, room_id: i32, opt_id: Option<u64>, client_info: ClientInfo) -> (usize, u64) {
        let mut id = opt_id.unwrap_or_else(rand::random);
        if let Some(room) = self.room_map.get_mut(&room_id) {
            loop {
                if room.contains_key(&id) {
                    id = rand::random();
                } else {
                    break;
                }
            }
            room.insert(id, client_info);
            let count = room.len();
            return (count, id);
        }
        // Create a new room for the first client
        let mut room: Room = HashMap::new();
        room.insert(id, client_info);
        let count = room.len();
        self.room_map.insert(room_id, room);
        (count, id)
    }
    // Get the number of clients in the room.
    fn count_clients_in_room(&self, room_id: i32) -> usize {
        let count = self.room_map.get(&room_id).map(|room| room.len()).unwrap_or(0);
        count
    }
    /** Send a chat message to all members. exclude - IDs of members to whom the message should not be sent.  */
    fn send_chat_message_to_clients(&mut self, room_id: i32, msg: &str, exclude: &[u64]) {
        let opt_room = self.take_room(room_id);
        if opt_room.is_none() {
            return;
        }
        let mut room = opt_room.unwrap();
        let command_srv = CommandSrv::Chat(ChatMsgSsn(msg.to_owned()));
        for (id, client_info) in room.drain() {
            let is_add = if exclude.contains(&id) {
                true
            } else {
                let is_connect = client_info.client.connected();
                if !is_connect {
                    #[rustfmt::skip]
                    debug!("send_chat_message_to_clients() room_id:{room_id}, user_name: {}, client.connected(): false",
                        client_info.name.clone());
                }
                // If the client has not yet broken the connection, then send a message.
                is_connect && client_info.client.try_send(command_srv.clone()).is_ok()
            };
            if is_add {
                self.add_client_to_room(room_id, Some(id), client_info);
            }
        }
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
        // Asynchronously subscribe to "LeaveRoom". (sending `BrokerIssue`.issue_system_async())
        self.subscribe_system_async::<LeaveRoom>(ctx);
    }
}

// ** Blocking clients in a room by name. (Session -> Server) **

impl Handler<BlockClient> for ChatWsServer {
    type Result = MessageResult<BlockClient>;

    fn handle(&mut self, msg: BlockClient, _ctx: &mut Self::Context) -> Self::Result {
        let mut is_in_chat = false;
        let BlockClient(room_id, user_name, is_block) = msg;
        if room_id <= i32::default() || user_name.len() == 0 {
            return MessageResult(is_in_chat);
        }
        // Get a chat room by its name.
        if let Some(room) = self.room_map.get(&room_id) {
            // Loop through all chat participants.
            for (_id, client_info) in room {
                // Checking the chat participant name with the name to block.
                if client_info.name.eq(&user_name) {
                    is_in_chat = true;
                    client_info.client.do_send(CommandSrv::Block(BlockSsn(is_block, is_in_chat)));
                }
            }
        }
        debug!("handler<BlockClient>() room_id:{room_id}, user_name: {user_name}, is_block:{is_block}, is_in_chat:{is_in_chat}");
        MessageResult(is_in_chat)
    }
}

// ** Count of clients in the room. (Session -> Server) **

impl Handler<CountMembers> for ChatWsServer {
    type Result = MessageResult<CountMembers>;

    fn handle(&mut self, msg: CountMembers, _ctx: &mut Self::Context) -> Self::Result {
        let CountMembers(room_id) = msg;
        let count = self.count_clients_in_room(room_id);
        debug!("handler<CountMembers>() room_id: {room_id}, room.len(): {count}");
        MessageResult(count)
    }
}

// ** Join the client to the chat room. (Session -> Server) **

impl Handler<JoinRoom> for ChatWsServer {
    type Result = MessageResult<JoinRoom>;

    fn handle(&mut self, msg: JoinRoom, _ctx: &mut Self::Context) -> Self::Result {
        let JoinRoom(room_id, user_name, client) = msg;
        let name = user_name.clone();
        let member = name.clone();
        // Add a client to the room. (count - number of members, id - new member ID)
        let (count, id) = self.add_client_to_room(room_id, None, ClientInfo { name, client });
        #[rustfmt::skip]
        let join_str = to_string(&JoinEWS { join: room_id, member, count, is_owner: None, is_blocked: None }).unwrap();
        debug!("handler<JoinRoom>() room_id: {room_id}, user_name: {user_name}, room.len(): {count}");
        // Send a chat message to all members.
        self.send_chat_message_to_clients(room_id, &join_str, &[id]);
        MessageResult((id, count))
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
            let opt_recipient = room.remove(&client_id);
            // Get the number of clients in the room.
            let count = room.len();
            let member = user_name.clone();
            #[rustfmt::skip]
            debug!("handler<LeaveRoom>() room_id: {room_id}, user_name: {user_name}, room.len(): {count}, room.remove(client_id): {} ",
                opt_recipient.is_some());
            let leave_str = to_string(&LeaveEWS {
                leave: room_id,
                member,
                count,
            })
            .unwrap();
            // Send a chat message to all members.
            self.send_chat_message_to_clients(room_id, &leave_str, &[]);

            if let Some(client_info) = opt_recipient {
                if client_info.client.connected() {
                    debug!("handler<LeaveRoom>() client_info.client.connected(): true");
                    client_info.client.do_send(CommandSrv::Chat(ChatMsgSsn(leave_str.to_owned())));
                }
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
        self.send_chat_message_to_clients(room_id, &msg_str, &[]);
    }
}

// ** -- **
