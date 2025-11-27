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

type Client = Recipient<CommandSrv>;

#[derive(Debug)]
pub struct ClientInfo {
    name: String,
    client: Client,
}

#[derive(Debug, Default)]
pub struct RoomInfo {
    owner_id: i32,
    map: HashMap<u32, ClientInfo>, // Map<client_id: u32, ClientInfo> u32::MAX = 4_294_967_295
}

/** Add a new client to the map of all clients. -> ("id" - new client ID, "count" - number of clients) */
fn add_client_to_map(client_map: &mut HashMap<u32, ClientInfo>, client_info: ClientInfo) -> (u32, usize) {
    let mut client_id = rand::random();
    loop {
        if client_map.contains_key(&client_id) {
            client_id = rand::random();
        } else {
            break;
        }
    }
    client_map.insert(client_id, client_info);
    let count = client_map.len();
    (client_id, count)
}
/** Remove a client from the map of all clients of this room. */
fn remove_client_from_map(client_map: &mut HashMap<u32, ClientInfo>, client_id: u32) -> Option<ClientInfo> {
    client_map.remove(&client_id)
}

// ** ChatWsServer **

#[derive(Default)]
pub struct ChatWsServer {
    rooms_map: HashMap<i32, RoomInfo>,
    owners_map: HashMap<i32, HashSet<i32>>, // Map<owner_id: i32, Set<room_id: i32>>
}

/** Get a room by ID (or create a new room) from the map of all rooms. */
fn get_or_create_room(rooms_map: &mut HashMap<i32, RoomInfo>, room_id: i32, owner_id: i32) -> &mut RoomInfo {
    if !rooms_map.contains_key(&room_id) {
        let mut room_info = RoomInfo::default();
        room_info.owner_id = owner_id;
        rooms_map.insert(room_id, room_info);
    }
    rooms_map.get_mut(&room_id).unwrap()
}
/** Remove room from all rooms map. */
fn remove_room(rooms_map: &mut HashMap<i32, RoomInfo>, room_id: i32) -> Option<RoomInfo> {
    rooms_map.remove(&room_id)
}

/** Add a new room to the set for the specified owner. */
fn add_room_to_owner(owners_map: &mut HashMap<i32, HashSet<i32>>, owner_id: i32, room_id: i32) {
    if let Some(room_id_set) = owners_map.get_mut(&owner_id) {
        if !room_id_set.contains(&room_id) {
            room_id_set.insert(room_id);
        }
    } else {
        owners_map.insert(owner_id, HashSet::from([room_id]));
    }
}
/** Remove a room from the set for the specified owner. */
fn remove_room_from_owner(owners_map: &mut HashMap<i32, HashSet<i32>>, owner_id: i32, room_id: i32) {
    if let Some(room_id_set) = owners_map.get_mut(&owner_id) {
        room_id_set.remove(&room_id);
        if room_id_set.len() == 0 {
            owners_map.remove(&owner_id);
        }
    }
}

impl ChatWsServer {
    // Take up room for changes.
    /*fn take_room(&mut self, room_id: i32) -> Option<Room> {
        let room = self.room_map.get_mut(&room_id)?;
        let room = std::mem::take(room);
        Some(room)
    }*/
    /** Add a client to the room. ("count" - number of members, "id" - new member ID) */
    /*fn add_client_to_room(&mut self, room_id: i32, opt_id: Option<u32>, client_info: ClientInfo) -> (usize, u32) {
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

    /** Get the number of clients in the room. */
    fn count_clients_in_room(&self, room_id: i32) -> usize {
        self.rooms_map.get(&room_id).map(|room| room.map.len()).unwrap_or(0)
    }
    /** Send a message to all participants. (exclude - IDs of members to whom the message should not be sent.) */
    fn send_message_to_clients(&mut self, room_id: i32, msg: &str, exclude: &[u32]) {
        let opt_room_info = self.rooms_map.get_mut(&room_id);
        if opt_room_info.is_none() {
            return;
        }
        let room_info = opt_room_info.unwrap();
        let command_srv = CommandSrv::Chat(ChatMsgSsn(msg.to_owned()));
        let mut buff_ids_to_delete: Vec<u32> = Vec::new();
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
            // Remove a client from the map of all clients of this room.
            remove_client_from_map(&mut room_info.map, client_id);
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
        if let Some(room_info) = self.rooms_map.get(&room_id) {
            // Loop through all chat participants.
            for (_id, client_info) in &room_info.map {
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
        let BlockUser(owner_id, user_name, is_block) = msg;
        if user_name.len() == 0 {
            return MessageResult(is_in_chat);
        }
        let room_id_set: HashSet<i32> = match self.owners_map.get(&owner_id) {
            Some(room_ids) => room_ids.clone(),
            None => HashSet::new(),
        };
        is_in_chat = true;
        let mut is_exit = false;

        for room_id in room_id_set {
            if let Some(room_info) = self.rooms_map.get(&room_id) {
                for (_client_id, client_info) in &room_info.map {
                    // Checking the chat participant's name against the name to be unblocked/blocked.
                    if client_info.name.eq(&user_name) {
                        client_info.client.do_send(CommandSrv::Block(BlockSsn(is_block, is_in_chat)));
                        is_exit = true;
                        break;
                    }
                }
            }
            if is_exit {
                break;
            }
        }
        debug!("handler<BlockUser>() owner_id: {owner_id}, user_name: {user_name}, is_block:{is_block}, is_in_chat:{is_in_chat}");
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
        let JoinRoom(room_id, owner_id, user_name, client) = msg;
        let name = user_name.clone();
        let member = name.clone();
        // Get a room by ID (or create a new room) from the map of all rooms.
        let room_info = get_or_create_room(&mut self.rooms_map, room_id, owner_id);
        // Add a new room for the specified owner.
        add_room_to_owner(&mut self.owners_map, owner_id, room_id);
        // Add a new client to the room.
        let (id, count) = add_client_to_map(&mut room_info.map, ClientInfo { name, client });
        #[rustfmt::skip]
        let join_str = to_string(&JoinEWS { join: room_id, member, count, is_owner: None, is_blocked: None }).unwrap();
        debug!("handler<JoinRoom>() room_id: {room_id}, user_name: {user_name}, room.len(): {count} Ok!");
        // Send a chat message to all members.
        self.send_message_to_clients(room_id, &join_str, &[id]);
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
        let mut opt_owner_id: Option<i32> = None;
        // Get a room by its ID.
        if let Some(room_info) = self.rooms_map.get_mut(&room_id) {
            // Remove a client from the map of all clients of this room.
            let opt_recipient = remove_client_from_map(&mut room_info.map, client_id);
            // Get the number of clients in the room.
            let count = room_info.map.len();
            let member = user_name.clone();
            #[rustfmt::skip]
            debug!("handler<LeaveRoom>() room_id: {room_id}, user_name: {user_name}, room.len(): {count}, client_id: {client_id}");
            #[rustfmt::skip]
            let leave_str = to_string(&LeaveEWS { leave: room_id, member, count }).unwrap();

            // If there are no clients left for a given room, it must be removed from the map of all rooms.
            if count == 0 {
                opt_owner_id = Some(room_info.owner_id);
            } else {
                // If count > 0 then send a chat message to all members.
                self.send_message_to_clients(room_id, &leave_str, &[]);
            }

            if let Some(client_info) = opt_recipient {
                if client_info.client.connected() {
                    client_info.client.do_send(CommandSrv::Chat(ChatMsgSsn(leave_str.to_owned())));
                }
            }
        }
        if let Some(owner_id) = opt_owner_id {
            // If there are no clients left in the room, then delete this room from all rooms map.
            remove_room(&mut self.rooms_map, room_id);
            // Remove a room from the set for the specified owner.
            remove_room_from_owner(&mut self.owners_map, owner_id, room_id);
        }
    }
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
