use actix::prelude::*;

// ** Blocking clients in a room by name. (Session -> Server) **
#[derive(Debug, Clone, Message)]
#[rtype(result = "()")] // amount of blocked members // MAX 4_294_967_295u32
pub struct BlockingClients(
    pub String, // room_name
    pub String, // client_name
    pub bool,   // is_blocked
);

// ** Send a block to the client in the room. (Server -> Session) **
#[derive(Debug, Clone, Message)]
#[rtype(result = "()")]
pub struct BlockingSsn(
    pub bool, // is_blocked
);

// ** Send a chat message to all clients in the room. (Server -> Session) **
#[derive(Debug, Clone, Message)]
#[rtype(result = "()")]
pub struct ChatMessageSsn(
    pub String, // message
);

// ** Commands that have one handler. (Session -> Server) **
#[derive(Debug, Clone, Message)]
#[rtype(result = "()")]
pub enum CommandSrv {
    Blocking(BlockingSsn),
    Chat(ChatMessageSsn),
}
/*
// ** Count of clients in the room. **
#[derive(Clone, Message)]
#[rtype(result = "usize")]
pub struct CountMembers(pub String);
*/
// ** Join the client to the chat room. (Session -> Server) **
#[derive(Clone, Message)]
#[rtype(result = "u64")] // id client
pub struct JoinRoomSrv(
    pub String,                // room_name
    pub Option<String>,        // client_name
    pub Recipient<CommandSrv>, // client_session: SessionCommand
);

// ** Leave the client from the chat room. (Session -> Server) **
#[derive(Clone, Message)]
#[rtype(result = "()")]
pub struct LeaveRoomSrv(
    pub String,         // room_name
    pub u64,            // id client
    pub Option<String>, // client_name
);
/*
// ** Send a message to everyone in the chat room. (Session -> Server) **
#[derive(Clone, Message)]
#[rtype(result = "()")]
pub struct SrvSendMessage(
    pub String, // room_name
    pub u64,    // id client
    pub String, // message
);
*/
