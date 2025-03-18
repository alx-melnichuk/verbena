use actix::prelude::*;

// ** Block the client in the room. **
#[derive(Clone, Message)]
#[rtype(result = "()")]
pub struct Block(
    pub String, // room_name
    pub String, // client_name
);

#[derive(Clone, Message)]
#[rtype(result = "()")]
pub struct ChatMessage(
    pub String, // message
);

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
);
// ** Leave the client from the chat room. **
#[derive(Clone, Message)]
#[rtype(result = "()")]
pub struct LeaveRoom(
    pub String,         // room_name
    pub u64,            // id client
    pub Option<String>, // client_name
);

// ** Send a message to everyone in the chat room. **

#[derive(Clone, Message)]
#[rtype(result = "()")]
pub struct SendMessage(
    pub String, // room_name
    pub u64,    // id client
    pub String, // message
);
