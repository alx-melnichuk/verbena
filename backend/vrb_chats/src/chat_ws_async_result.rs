
// * * * * Handler for asynchronous response to the "error" command. * * * *

pub struct AsyncResultError(
    pub u16,    // err
    pub String, // code
    pub String, // message
);

// * * * * Handler for asynchronous response to the "BlockClient" event * * * *

pub struct AsyncResultBlockClient(
    pub i32,    // room_id
    pub bool,   // is_block
    pub String, // blocked_name
);

// * * * *  _  * * * *
