use std::slice::Iter;
use std::{collections::HashMap, fmt};

use serde::{Deserialize, Serialize};
use serde_json;

pub const MISSING_STARTING_CURLY_BRACE: &str = "Serialization: missing \"{\".";
pub const MISSING_ENDING_CURLY_BRACE: &str = "Serialization: missing \"}\".";
pub const UNKNOWN_COMMAND: &str = "unknown command: ";
pub const SERIALIZATION: &str = "Serialization: ";

// ** Types of events that are transmitted over a websocket. **

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum EWSType {
    Block,
    Count,
    Echo,
    Err,
    Join,
    Leave,
    Msg,
    MsgCut,
    MsgPut,
    Name,
    Unblock,
}

impl EWSType {
    pub fn iterator() -> Iter<'static, EWSType> {
        static LIST: [EWSType; 11] = [
            EWSType::Block,
            EWSType::Count,
            EWSType::Echo,
            EWSType::Err,
            EWSType::Join,
            EWSType::Leave,
            EWSType::Msg,
            EWSType::MsgCut,
            EWSType::MsgPut,
            EWSType::Name,
            EWSType::Unblock,
        ];
        LIST.iter()
    }
    pub fn parse(data: &str) -> Option<Self> {
        let mut result: Option<Self> = None;
        let data_str = data.to_lowercase();
        for et in EWSType::iterator() {
            if data_str.eq(&et.to_string().to_lowercase()) {
                result = Some(et.clone());
                break;
            }
        }
        result
    }
}

impl fmt::Display for EWSType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", serde_json::to_string(&self).unwrap().replace("\"", ""))
    }
}

// ** Parse the type and data for a web socket event. **

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EventWS {
    et: EWSType,
    params: HashMap<String, serde_json::Value>,
}

impl EventWS {
    pub fn new(et: EWSType, params_opt: Option<HashMap<String, serde_json::Value>>) -> Self {
        EventWS {
            et,
            params: params_opt.unwrap_or(HashMap::new()),
        }
    }
    // Parse input data of ws event.
    pub fn parsing(event: &str) -> Result<Self, String> {
        if !event.starts_with('{') {
            return Err(MISSING_STARTING_CURLY_BRACE.to_string());
        }
        if !event.ends_with('}') {
            return Err(MISSING_ENDING_CURLY_BRACE.to_string());
        }
        // Get the name of the first tag.
        let mut buf = event.split("\"");
        buf.next();
        let first_tag = buf.next().unwrap_or("");

        let res_ews_type = EWSType::parse(first_tag);
        if let None = res_ews_type {
            return Err(format!("{}{:?}", UNKNOWN_COMMAND, event).to_string());
        }
        let ews_type = res_ews_type.unwrap();

        let mut params: HashMap<String, serde_json::Value> = HashMap::new();
        // Parse the input data.
        let parsed = serde_json::from_str::<serde_json::Value>(event)
            .map_err(|e| format!("{}{}", SERIALIZATION, e.to_string()))?;

        let obj = parsed.as_object().unwrap().clone();
        for (key, val) in obj.iter() {
            // let value = if val.is_string() { val.as_str().unwrap().to_string() } else { val.to_string() };
            params.insert(key.clone(), val.clone());
        }
        Ok(EventWS::new(ews_type, Some(params)))
    }

    pub fn ews_type(&self) -> EWSType {
        self.et.clone()
    }

    pub fn get_string(&self, name: &str) -> Option<String> {
        let mut result: Option<String> = None;
        if let Some(value) = self.params.get(name).map(|s| s.clone()) {
            if value.is_string() {
                result = value.as_str().map(|s| s.to_string());
            }
        }
        result
    }

    pub fn get_i32(&self, name: &str) -> Option<i32> {
        let mut result: Option<i32> = None;
        if let Some(value) = self.params.get(name).map(|s| s.clone()) {
            if let Some(res) = parse_json_to_i32(&value).ok() {
                result = Some(res);
            }
        }
        result
    }
}

fn parse_json_to_i32(json_value: &serde_json::Value) -> Result<i32, &'static str> {
    // Check if the JSON value is of type Number.
    if let Some(number) = json_value.as_i64() {
        // Try to convert the i64 number to i32.
        // If the number is within the range of i32, return it as i32.
        // Otherwise, return an error string.
        if number >= i32::MIN as i64 && number <= i32::MAX as i64 {
            Ok(number as i32)
        } else {
            Err("JSON value is out of range for i32.")
        }
    } else {
        Err("JSON value is not a number.")
    }
}

// ** **

// ** Block clients in a room by name. **
#[derive(Serialize, Deserialize, Clone)]
pub struct BlockEWS {
    pub block: String,
    pub count: u32,
}

// ** Count of clients in the room. **
#[derive(Serialize, Deserialize, Clone)]
pub struct CountEWS {
    pub count: usize,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct EchoEWS {
    pub echo: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ErrEWS {
    pub err: String,
}

// ** Join the client to the chat room. **
#[derive(Serialize, Deserialize, Clone)]
pub struct JoinEWS {
    pub join: i32,
    pub member: String,
    pub count: usize,
}

// ** Leave the client from the chat room. **
#[derive(Serialize, Deserialize, Clone)]
pub struct LeaveEWS {
    pub leave: i32,
    pub member: String,
    pub count: usize,
}

// ** Send a text message to all clients in the room. **
#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MsgEWS {
    pub msg: String,
    pub id: i32,
    pub member: String,
    pub date: String,
    pub is_edt: bool,
    pub is_rmv: bool,
}

// ** Send a delete message to all chat members. **
#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MsgCutEWS {
    pub msg_cut: String,
    pub id: i32,
}

// ** Send a correction to the message to everyone in the chat. **
#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MsgPutEWS {
    pub msg_put: String,
    pub id: i32,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct NameEWS {
    pub name: String, // user_name
    pub id: i32,      // user id from users table
}

// ** Unblock clients in a room by name. **
#[derive(Serialize, Deserialize, Clone)]
pub struct UnblockEWS {
    pub unblock: String,
    pub count: u32,
}

// ** **

#[cfg(test)]
mod tests {

    use super::*;

    // ** EventWS **

    #[test]
    fn test_eventws_parse_1() {
        for et in EWSType::iterator() {
            let et_s = et.to_string().to_lowercase();
            let str = format!("{}_a", et_s);
            let res = EWSType::parse(&str);
            assert_eq!(res, None, "EWSType={:?}", et);
        }
    }
    #[test]
    fn test_eventws_parse_2() {
        for et in EWSType::iterator() {
            let et_s = et.to_string().to_lowercase();
            let res = EWSType::parse(&et_s);
            assert_eq!(res, Some(et.clone()), "EWSType={:?}", et);
        }
    }
}
