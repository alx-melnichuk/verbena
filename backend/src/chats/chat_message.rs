use actix::prelude::*;

// ** Blocking clients in a room by name. (Session -> Server) **
/*#[derive(Clone, Message)]
#[rtype(result = "u32")] // Count of chat room members. // MAX 4_294_967_295u32
pub struct BlockClients(
    pub String, // room_name
    pub String, // client_name
    pub bool,   // is_blocked
);*/

// ** Send a block to the client in the room. (Server -> Session) **
/*#[derive(Clone, Message)]
#[rtype(result = "()")]
pub struct BlockSsn(
    pub bool, // is_blocked
);*/

// ** Send a chat message to all clients in the room. (Server -> Session) **
#[derive(Clone, Message)]
#[rtype(result = "()")]
pub struct ChatMsgSsn(
    pub String, // message
);

// ** Commands that have one handler. (Session -> Server) **
#[derive(Clone, Message)]
#[rtype(result = "()")]
pub enum CommandSrv {
    /* Block(BlockSsn), */
    Chat(ChatMsgSsn),
}

// ** Count of clients in the room. (Session -> Server) **
/*#[derive(Clone, Message)]
#[rtype(result = "usize")] // MAX 18_446_744_073_709_551_615usize
pub struct CountMembers(
    pub String, // room_name
);*/

// ** Join the client to the chat room. (Session -> Server) **
#[derive(Clone, Message)]
#[rtype(result = "u64")] // id client  // MAX 18_446_744_073_709_551_615u64
pub struct JoinRoom(
    pub i32,                   // room_id
    pub Option<String>,        // client_name
    pub Recipient<CommandSrv>, // client_session: SessionCommand
);

// ** Leave the client from the chat room. (Session -> Server) **
#[derive(Clone, Message)]
#[rtype(result = "()")]
pub struct LeaveRoom(
    pub i32,            // room_id
    pub u64,            // id client
    pub Option<String>, // client_name
);

// ** Send a text message to all clients in the room. (Server -> Session) **
/*#[derive(Clone, Message)]
#[rtype(result = "()")]
pub struct SendMessage(
    pub String, // room_name
    pub String, // message
);*/

// **  (Session -> Session) **
/*#[derive(Clone, Message)]
#[rtype(result = "(i32)")]
pub struct SaveMessageResult(
    pub i32, // id message
);*/
