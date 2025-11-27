use actix::prelude::*;
use actix_web_actors::ws::CloseReason;

// ** Blocking client in a room by name. (Session -> Server) **
#[derive(Debug, Clone, Message)]
#[rtype(result = "bool")] // is_in_chat
pub struct BlockClient(
    pub i32,    // room_id
    pub String, // client_name
    pub bool,   // is_block
);

// ** Send a block to the client in the room. (Server -> Session) **
#[derive(Debug, Clone, Message)]
#[rtype(result = "()")]
pub struct BlockSsn(
    pub bool, // is_block
    pub bool, // is_in_chat
);

// ** Blocking a user by their nickname. (Session -> Server) **
#[derive(Debug, Clone, Message)]
#[rtype(result = "bool")] // is_in_chat
pub struct BlockUser(
    pub i32,    // owner_id
    pub String, // client_name
    pub bool,   // is_block
);

// ** Send a chat message to all clients in the room. (Server -> Session) **
#[derive(Debug, Clone, Message)]
#[rtype(result = "()")]
pub struct ChatMsgSsn(
    pub String, // message
);

// ** Commands that have one handler. (Session -> Server) **
#[derive(Debug, Clone, Message)]
#[rtype(result = "()")]
pub enum CommandSrv {
    Block(BlockSsn),
    Chat(ChatMsgSsn),
    CloseAndStop(Option<CloseReason>),
}

// ** Count of clients in the room. (Session -> Server) **
#[derive(Debug, Clone, Message)]
#[rtype(result = "usize")] // usize::MAX = 18_446_744_073_709_551_615
pub struct CountMembers(
    pub i32, // room_id
);

// ** Join the client to the chat room. (Session -> Server) **
#[derive(Debug, Clone, Message)]
#[rtype(result = "(u32, usize)")] // (client_id, count)  // u32::MAX = 4_294_967_295
pub struct JoinRoom(
    pub i32,                   // room_id
    pub i32,                   // owner_id
    pub String,                // client_name
    pub Recipient<CommandSrv>, // client_session: SessionCommand
);

// ** Leave the client from the chat room. (Session -> Server) **
#[derive(Debug, Clone, Message)]
#[rtype(result = "()")]
pub struct LeaveRoom(
    pub i32,    // room_id
    pub u32,    // client_id
    pub String, // client_name
);

// ** Send a text message to all clients in the room. (Server -> Session) **
#[derive(Debug, Clone, Message)]
#[rtype(result = "()")]
pub struct SendMessage(
    pub i32,    // room_id
    pub String, // message
);
