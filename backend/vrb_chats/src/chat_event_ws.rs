use std::{collections::HashMap, fmt, slice::Iter};

use chrono::SecondsFormat;
use serde::{Deserialize, Serialize};
use serde_json;

use crate::chat_message_models::ChatMessage;

pub const MISSING_STARTING_CURLY_BRACE: &str = "Serialization: missing \"{\".";
pub const MISSING_ENDING_CURLY_BRACE: &str = "Serialization: missing \"}\".";
pub const UNKNOWN_COMMAND: &str = "unknown command: ";
pub const SERIALIZATION: &str = "Serialization: ";

// ** Types of events that are transmitted over a websocket. **

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum EWSType {
    Block,
    Close,
    Count,
    Echo,
    Err,
    Join,
    Leave,
    Msg,
    MsgCut,
    MsgPut,
    MsgRmv,
    Name,
    PrmBool,
    PrmInt,
    PrmStr,
    Unblock,
}

impl EWSType {
    pub fn iterator() -> Iter<'static, EWSType> {
        static LIST: [EWSType; 16] = [
            EWSType::Block,
            EWSType::Close,
            EWSType::Count,
            EWSType::Echo,
            EWSType::Err,
            EWSType::Join,
            EWSType::Leave,
            EWSType::Msg,
            EWSType::MsgCut,
            EWSType::MsgPut,
            EWSType::MsgRmv,
            EWSType::Name,
            EWSType::PrmBool,
            EWSType::PrmInt,
            EWSType::PrmStr,
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

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
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
        #[rustfmt::skip]
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

    pub fn get_bool(&self, name: &str) -> Option<bool> {
        if let Some(value) = self.params.get(name).map(|s| s.clone()) {
            value.as_bool()
        } else {
            None
        }
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
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct BlockEWS {
    pub block: String,
    pub is_in_chat: bool, // The user is in chat now.
}

// ** Count of clients in the room. **
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct CountEWS {
    pub count: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct EchoEWS {
    pub echo: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct ErrEWS {
    pub err: u16,
    pub code: String,
    pub message: String,
}

impl fmt::Display for ErrEWS {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", serde_json::to_string(&self).unwrap())
    }
}

impl ErrEWS {
    #[rustfmt::skip]
    pub fn new(err: u16, code: String, message: String) -> Self {
        ErrEWS { err, code, message }
    }
}

// ** Join the client to the chat room. **
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct JoinEWS {
    pub join: i32,
    pub member: String,
    pub count: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_owner: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_blocked: Option<bool>,
}

// ** Leave the client from the chat room. **
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct LeaveEWS {
    pub leave: i32,
    pub member: String,
    pub count: usize,
}

// ** Send a text message to all clients in the room. **
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MsgEWS {
    pub msg: String,
    pub id: i32,
    pub member: String,
    pub date: String,             // DateTime<Utc>
    pub date_edt: Option<String>, // DateTime<Utc>
    pub date_rmv: Option<String>, // DateTime<Utc>
}

impl From<ChatMessage> for MsgEWS {
    fn from(chat_message: ChatMessage) -> Self {
        MsgEWS {
            msg: chat_message.msg.unwrap_or_default(),
            id: chat_message.id,
            member: chat_message.user_name.clone(),
            date: chat_message.date_created.to_rfc3339_opts(SecondsFormat::Millis, true),
            date_edt: chat_message.date_changed.map(|v| v.to_rfc3339_opts(SecondsFormat::Millis, true)),
            date_rmv: chat_message.date_removed.map(|v| v.to_rfc3339_opts(SecondsFormat::Millis, true)),
        }
    }
}

// ** Send a message about deleting text to all chat members. **
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MsgCutEWS {
    pub msg_cut: String,
    pub id: i32,
}

// ** Send a correction to the message to everyone in the chat. **
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MsgPutEWS {
    pub msg_put: String,
    pub id: i32,
}

/** Send a permanent deletion message to all chat members. */
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MsgRmvEWS {
    pub msg_rmv: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct NameEWS {
    pub name: String, // user_name
}

// ** Send a parameter with the name and type boolean. **
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct PrmBoolEWS {
    pub prm_bool: String, // Parameter name.
    pub val_bool: bool, // Parameter value.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_owner: Option<bool>, // Indicates that the chat was sent by the owner.
}

// ** Send a parameter with the name and type integer (i32). **
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct PrmIntEWS {
    pub prm_int: String, // Parameter name.
    pub val_int: i32, // Parameter value.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_owner: Option<bool>, // Indicates that the chat was sent by the owner.
}

// ** Send a parameter with the name and type string. **
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct PrmStrEWS {
    pub prm_str: String, // Parameter name.
    pub val_str: String, // Parameter value.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_owner: Option<bool>, // Indicates that the chat was sent by the owner.
}

// ** Unblock clients in a room by name. **
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct UnblockEWS {
    pub unblock: String,
    pub is_in_chat: bool, // The user is in chat now.
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
