use std::collections::{HashMap, HashSet};

use actix::prelude::*;
use actix_broker::BrokerSubscribe;
use log::debug;
use rand;
use serde_json::to_string;

use crate::{
    chat_event_ws::{JoinEWS, LeaveEWS},
    chat_message::{BlockClient, BlockSsn, BlockUser, ChatMsgSsn, CommandSrv, CountMembers, JoinRoom, LeaveRoom, SendMessage},
};

type Client = Recipient<CommandSrv>; // ChatMessage

#[derive(Debug)]
pub struct ClientInfo {
    name: String,
    client: Client,
}

#[derive(Debug, Default)]
pub struct RoomInfo {
    owner_id: i32,
    map: HashMap<u64, ClientInfo>,
}

impl RoomInfo {
    pub fn add_client(&mut self, opt_client_id: Option<u64>, client_info: ClientInfo) -> (u64, usize) {
        let mut client_id = opt_client_id.unwrap_or_else(rand::random);
        loop {
            if self.map.contains_key(&client_id) { client_id = rand::random(); } else { break; }
        }
        self.map.insert(client_id, client_info);
        let count = self.map.len();
        (client_id, count)
    }
    pub fn remove_client(&mut self, client_id: u64) -> Option<ClientInfo> {
        self.map.remove(&client_id)
    }
    pub fn get_owner_id(&self) -> i32 {
        self.owner_id
    }
}

type Room = HashMap<u64, ClientInfo>;

// ** ChatWsServer **

#[derive(Default)]
pub struct ChatWsServer {
    room_map: HashMap<i32, Room>,
    rooms2_map: HashMap<i32, RoomInfo>,
    owners2_map: HashMap<i32, HashSet<i32>>,
}

impl ChatWsServer {
    // Take up room for changes.
    /*fn take_room(&mut self, room_id: i32) -> Option<Room> {
        let room = self.room_map.get_mut(&room_id)?;
        let room = std::mem::take(room);
        Some(room)
    }*/
    /** Add a client to the room. ("count" - number of members, "id" - new member ID) */
    /*fn add_client_to_room(&mut self, room_id: i32, opt_id: Option<u64>, client_info: ClientInfo) -> (usize, u64) {
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
    }*/
    // Get the number of clients in the room.
    /*fn count_clients_in_room(&self, room_id: i32) -> usize {
        let count = self.room_map.get(&room_id).map(|room| room.len()).unwrap_or(0);
        count
    }*/
    /** Send a chat message to all members. exclude - IDs of members to whom the message should not be sent.  */
    /*fn send _chat_message_to_clients(&mut self, room_id: i32, msg: &str, exclude: &[u64]) {
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
                    let user_name = client_info.name.clone();
                    debug!("send chat_message_to_clients() room_id:{room_id}, user_name: {user_name}, client.connected(): false");
                }
                // If the client has not yet broken the connection, then send a message.
                is_connect && client_info.client.try_send(command_srv.clone()).is_ok()
            };
            if is_add {
                self.add_client_to_room(room_id, Some(id), client_info);
            }
        }
    }*/
    /// Create a new room.
    fn get_or_create_room(&mut self, room_id: i32, owner_id: i32) -> &mut RoomInfo {
        if !self.rooms2_map.contains_key(&room_id) {
            let mut room_info = RoomInfo::default();
            room_info.owner_id = owner_id;
            self.rooms2_map.insert(room_id, room_info);

            if let Some(owner_set) = self.owners2_map.get_mut(&owner_id) {
                if !owner_set.contains(&room_id) {
                    owner_set.insert(room_id);
                }
            } else {
                self.owners2_map.insert(owner_id, HashSet::from([room_id]));
            }
        }
        self.rooms2_map.get_mut(&room_id).unwrap()
    }
    fn get_mut_room(&mut self, room_id: i32) -> Option<&mut RoomInfo> {
        self.rooms2_map.get_mut(&room_id)
    }
    /// Delete a room.
    fn remove_room(&mut self, room_id: i32) {
        if let Some(room_info) = self.rooms2_map.remove(&room_id) {
            let owner_id = room_info.get_owner_id();
            if let Some(owner_set) = self.owners2_map.get_mut(&owner_id) {
                owner_set.remove(&room_id);
                if owner_set.len() == 0 {
                    self.owners2_map.remove(&owner_id);
                }
            }
        }
    }
    // Take up room for changes.
    /*fn take_room_info(&mut self, room_id: i32) -> Option<RoomInfo> {
        let room_info = self.rooms2_map.get_mut(&room_id)?;
        let mut room_info2 = std::mem::take(room_info);
        room_info2.owner_id = room_info.owner_id;
        Some(room_info2)
    }*/
    /// Send a message to all participants. (exclude - IDs of members to whom the message should not be sent.)
    fn send_message_to_clients(&mut self, room_id: i32, msg: &str, exclude: &[u64]) {
        let opt_room_info = self.get_mut_room(room_id);
        if opt_room_info.is_none() {
            return;
        }
        let room_info = opt_room_info.unwrap();
        let command_srv = CommandSrv::Chat(ChatMsgSsn(msg.to_owned()));
        let mut buff_ids_to_delete: Vec<u64> = Vec::new();
        // Clears the map, returning all key-value pairs as an iterator. Keeps the allocated memory for reuse.
        for (client_id, client_info) in room_info.map.iter() {
            // The client is on the exception list and should not be sent the message. 
            let is_preserved = exclude.contains(&client_id)
                // The client did not close the connection.
                || (client_info.client.connected()
                    // Sending a message to the client was successful.
                    && client_info.client.try_send(command_srv.clone()).is_ok());

            // Save the ID of the client for whom the message was not sent successfully.
            if !is_preserved {
                buff_ids_to_delete.push(*client_id);
            }
        }
        for client_id in buff_ids_to_delete {
            room_info.map.remove(&client_id);
        }
    }
    /// Get the number of clients in the room.
    fn count_clients_in_room2(&self, room_id: i32) -> usize {
        self.rooms2_map.get(&room_id).map(|room_info| room_info.map.len()).unwrap_or(0)
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
                    break;
                }
            }
        }
        debug!("handler<BlockClient>() room_id:{room_id}, user_name: {user_name}, is_block:{is_block}, is_in_chat:{is_in_chat}");
        MessageResult(is_in_chat)
    }
}

// ** Blocking a user by their nickname. (Session -> Server) **

impl Handler<BlockUser> for ChatWsServer {
    type Result = MessageResult<BlockUser>;

    fn handle(&mut self, msg: BlockUser, _ctx: &mut Self::Context) -> Self::Result {
        let mut is_in_chat = false;
        let BlockUser(_owner_name, user_name, is_block) = msg;
        eprintln!("\n_handler<BlockUser>() user_name: {user_name}, is_block:{is_block}\n");
        if user_name.len() == 0 {
            return MessageResult(is_in_chat);
        }
        let mut opt_recipient: Option<Recipient<CommandSrv>> = None;
        for room in self.room_map.values() {
            // Loop through all chat participants.
            for (_id, client_info) in room {
                eprintln!("_handler<BlockUser>() client_info.name:{}", &client_info.name); // #
                // Checking the chat participant name with the name to block.
                if client_info.name.eq(&user_name) {
                    is_in_chat = true;
                    opt_recipient = Some(client_info.client.clone());
                    // client_info.client.do_send(CommandSrv::Block(BlockSsn(is_block, is_in_chat)));
                    break;
                }
            }
            if opt_recipient.is_some() {
                break;
            }
        }
        if let Some(recipient) = opt_recipient {
            recipient.do_send(CommandSrv::Block(BlockSsn(is_block, is_in_chat)));
        }
        debug!("handler<BlockUser>() user_name: {user_name}, is_block:{is_block}, is_in_chat:{is_in_chat}");
        MessageResult(is_in_chat)
    }
}

// ** Count of clients in the room. (Session -> Server) **

impl Handler<CountMembers> for ChatWsServer {
    type Result = MessageResult<CountMembers>;

    fn handle(&mut self, msg: CountMembers, _ctx: &mut Self::Context) -> Self::Result {
        let CountMembers(room_id) = msg;
        let count = self.count_clients_in_room2(room_id);
        debug!("handler<CountMembers>() room_id: {room_id}, room.len(): {count}");
        MessageResult(count)
    }
}

// ** Join the client to the chat room. (Session -> Server) **

impl Handler<JoinRoom> for ChatWsServer {
    type Result = MessageResult<JoinRoom>;

    fn handle(&mut self, msg: JoinRoom, _ctx: &mut Self::Context) -> Self::Result {
        let JoinRoom(room_id, owner_id, user_name, client) = msg;
        let name = user_name.clone();
        let member = name.clone();
        // Create a new room with "room_id" or return an existing one.
        let room_info = self.get_or_create_room(room_id, owner_id);
        // Add a new client to the room. ("count" - number of members, "id" - new member ID)
        let (id, count) = room_info.add_client(None, ClientInfo { name, client });
        #[rustfmt::skip]
        let join_str = to_string(&JoinEWS { join: room_id, member, count, is_owner: None, is_blocked: None }).unwrap();
        debug!("handler<JoinRoom>() room_id: {room_id}, user_name: {user_name}, room.len(): {count} Ok!");
        // Send a chat message to all members.
        // #self.send_ chat_message_to_clients(room_id, &join_str, &[id]);
        self.send_message_to_clients(room_id, &join_str, &[id]);
        
        dbg!("handler<JoinRoom>()", &self.rooms2_map, "handler<JoinRoom>()", &self.owners2_map); // #
        MessageResult((id, count))
    }
    /*fn handle(&mut self, msg: JoinRoom, _ctx: &mut Self::Context) -> Self::Result {
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
    }*/
}

// ** Leave the client from the chat room. (Session -> Server) **

impl Handler<LeaveRoom> for ChatWsServer {
    type Result = ();

    fn handle(&mut self, msg: LeaveRoom, _ctx: &mut Self::Context) {
        let room_id = msg.0;
        let client_id = msg.1;
        let user_name = msg.2;
        // Get a room by its ID.
        if let Some(room_info) = self.get_mut_room(room_id) {
            // Remove client from this room.
            let opt_recipient = room_info.remove_client(client_id);
            // Get the number of clients in the room.
            let count = room_info.map.len();
            let member = user_name.clone();
            #[rustfmt::skip]
            debug!("handler<LeaveRoom>() room_id: {room_id}, user_name: {user_name}, room.len(): {count}, client_id: {client_id}");
            #[rustfmt::skip]
            let leave_str = to_string(&LeaveEWS { leave: room_id, member, count }).unwrap();
            
            if count > 0 { // Send a chat message to all members.
                // #self.send_ chat_message_to_clients(room_id, &leave_str, &[]);
                self.send_message_to_clients(room_id, &leave_str, &[]);
            } else { // If there are no clients left in the room, then delete this room.
                self.remove_room(room_id);
            }

            if let Some(client_info) = opt_recipient {
                if client_info.client.connected() {
                    client_info.client.do_send(CommandSrv::Chat(ChatMsgSsn(leave_str.to_owned())));
                }
            }
            dbg!("handler<LeaveRoom>()", &self.rooms2_map, "handler<LeaveRoom>()", &self.owners2_map); // #
        }
    }
    /*fn handle(&mut self, msg: LeaveRoom, _ctx: &mut Self::Context) {
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
            #[rustfmt::skip]
            let leave_str = to_string(&LeaveEWS { leave: room_id, member, count }).unwrap();
            // Send a chat message to all members.
            self.send_chat_message_to_clients(room_id, &leave_str, &[]);

            if let Some(client_info) = opt_recipient {
                if client_info.client.connected() {
                    debug!("handler<LeaveRoom>() client_info.client.connected(): true");
                    client_info.client.do_send(CommandSrv::Chat(ChatMsgSsn(leave_str.to_owned())));
                }
            }
        }
    }*/
}

// ** Send a text message to all clients in the room. (Server -> Session) **

impl Handler<SendMessage> for ChatWsServer {
    type Result = ();

    fn handle(&mut self, msg: SendMessage, _ctx: &mut Self::Context) {
        let SendMessage(room_id, msg_str) = msg;
        // Send a chat message to all members.
        self.send_message_to_clients(room_id, &msg_str, &[]);
    }
}

// ** -- **
