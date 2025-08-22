
// ** Section: "Profiles" **

pub const USER1_ID: i32 = 1100;
pub const USER2_ID: i32 = 1101;
pub const USER3_ID: i32 = 1102;
pub const USER4_ID: i32 = 1103;

pub struct UserOrmTest {}

impl UserOrmTest {
    pub fn user_ids() -> Vec<i32> {
        vec![USER1_ID, USER2_ID, USER3_ID, USER4_ID]
    }    
}

// **  **
