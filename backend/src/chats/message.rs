use actix::prelude::*;

// ** Block clients in a room by name. **
#[derive(Debug, Clone, Message)]
#[rtype(result = "String")] // amount of blocked members
pub struct BlockMembers(
    pub String, // room_name
    pub String, // client_name
);

// ** Send a chat message to all clients in the room. **
#[derive(Debug, Clone, Message)]
#[rtype(result = "()")]
pub struct ChatMessage(
    pub String, // message
);
// ** Commands that have one handler. (Session -> Server) **
#[derive(Debug, Clone, Message)]
#[rtype(result = "()")]
pub enum SrvCommand {
    Block(BlockMembers),
    Chat(ChatMessage),
}

// ** Count of clients in the room. **
#[derive(Clone, Message)]
#[rtype(result = "usize")]
pub struct CountMembers(pub String);

// ** Join the client to the chat room. **
#[derive(Clone, Message)]
#[rtype(result = "u64")] // id client
pub struct JoinRoom(
    pub String,                 // room_name
    pub Option<String>,         // client_name
    pub Recipient<ChatMessage>, // client
    pub Recipient<SrvCommand>,  // client_session: SessionCommand
);

// ** Leave the client from the chat room. (Session -> Server) **
#[derive(Clone, Message)]
#[rtype(result = "()")]
pub struct SrvLeaveRoom(
    pub String,         // room_name
    pub u64,            // id client
    pub Option<String>, // client_name
);

// ** Send a message to everyone in the chat room. (Session -> Server) **
#[derive(Clone, Message)]
#[rtype(result = "()")]
pub struct SrvSendMessage(
    pub String, // room_name
    pub u64,    // id client
    pub String, // message
);
