use std::slice::Iter;
use std::{collections::HashMap, fmt};

use serde::{Deserialize, Serialize};
use serde_json;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum WSEventType {
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
}

impl WSEventType {
    pub fn iterator() -> Iter<'static, WSEventType> {
        static LIST: [WSEventType; 10] = [
            WSEventType::Block,
            WSEventType::Count,
            WSEventType::Echo,
            WSEventType::Err,
            WSEventType::Join,
            WSEventType::Leave,
            WSEventType::Msg,
            WSEventType::MsgCut,
            WSEventType::MsgPut,
            WSEventType::Name,
        ];
        LIST.iter()
    }

    pub fn parse(data: &str) -> Option<Self> {
        let mut result: Option<Self> = None;
        let data_str = data.to_lowercase();
        for et in WSEventType::iterator() {
            if data_str.eq(&et.to_string().to_lowercase()) {
                result = Some(et.clone());
                break;
            }
        }
        result
    }
}

impl fmt::Display for WSEventType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", serde_json::to_string(&self).unwrap().replace("\"", ""))
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct WSEvent {
    pub et: WSEventType,
    data_map: HashMap<String, String>,
}

impl WSEvent {
    pub fn new(et: WSEventType, data_map_opt: Option<HashMap<String, String>>) -> Self {
        WSEvent {
            et,
            data_map: data_map_opt.unwrap_or(HashMap::new()),
        }
    }
    // Parse input data of ws event.
    pub fn parsing(event: &str) -> Result<Self, String> {
        if !event.starts_with('{') {
            return Err("Serialization: missing \"{\".".to_string());
        }
        if !event.ends_with('}') {
            return Err("Serialization: missing \"}\".".to_string());
        }
        // Get the name of the first tag.
        let mut buf = event.split("\"");
        buf.next();
        let first_tag = buf.next().unwrap_or("");

        let res_event_type = WSEventType::parse(first_tag);
        if let None = res_event_type {
            return Err(format!("unknown command: {:?}", event).to_string());
        }
        let event_type = res_event_type.unwrap();

        let mut data_map: HashMap<String, String> = HashMap::new();
        // Parse the input data.
        let parsed = serde_json::from_str::<serde_json::Value>(event)
            .map_err(|e| format!("Serialization: {}", e.to_string()))?;

        let obj = parsed.as_object().unwrap().clone();
        for (key, val) in obj.iter() {
            let value = if val.is_string() {
                val.as_str().unwrap().to_string()
            } else {
                val.to_string()
            };
            data_map.insert(key.clone(), value);
        }
        Ok(WSEvent::new(event_type, Some(data_map)))
    }

    pub fn get(&self, name: &str) -> Option<String> {
        return self.data_map.get(name).map(|s| s.clone());
    }

    pub fn block(block: String, count: Option<u64>) -> String {
        serde_json::to_string(&WSEventBlock { block, count }).unwrap()
    }

    pub fn count(count: usize) -> String {
        serde_json::to_string(&WSEventCount { count }).unwrap()
    }

    pub fn echo(echo: String) -> String {
        serde_json::to_string(&WSEventEcho { echo }).unwrap()
    }

    pub fn err(err: String) -> String {
        serde_json::to_string(&WSEventErr { err }).unwrap()
    }

    pub fn name(name: String) -> String {
        serde_json::to_string(&WSEventName { name }).unwrap()
    }

    pub fn join(join: String, member: String, count: usize) -> String {
        serde_json::to_string(&WSEventJoin { join, member, count }).unwrap()
    }

    pub fn leave(leave: String, member: String, count: usize) -> String {
        serde_json::to_string(&WSEventLeave { leave, member, count }).unwrap()
    }

    pub fn msg(msg: String, member: String, date: String) -> String {
        serde_json::to_string(&WSEventMsg { msg, member, date }).unwrap()
    }

    pub fn msg_cut(msg_cut: String, member: String, date: String) -> String {
        serde_json::to_string(&WSEventMsgCut { msg_cut, member, date }).unwrap()
    }

    pub fn msg_put(msg_put: String, member: String, date: String) -> String {
        serde_json::to_string(&WSEventMsgPut { msg_put, member, date }).unwrap()
    }
}

// ** Block clients in a room by name. **
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct WSEventBlock {
    pub block: String,
    pub count: Option<u64>, // Amount of blocked clients.
}

// ** Count of clients in the room. **
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct WSEventCount {
    pub count: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct WSEventEcho {
    pub echo: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct WSEventErr {
    pub err: String,
}

// ** Join the client to the chat room. **
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct WSEventJoin {
    pub join: String,
    pub member: String,
    pub count: usize,
}

// ** Leave the client from the chat room. **
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct WSEventLeave {
    pub leave: String,
    pub member: String,
    pub count: usize,
}

// ** Send a message to everyone in the chat room. **
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct WSEventMsg {
    pub msg: String,
    pub member: String,
    pub date: String,
}

// ** Send a delete message to all chat members. **
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct WSEventMsgCut {
    pub msg_cut: String,
    pub member: String,
    pub date: String,
}

// ** Send a correction to the message to everyone in the chat. **
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct WSEventMsgPut {
    pub msg_put: String,
    pub member: String,
    pub date: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct WSEventName {
    pub name: String,
}

// ** **

#[cfg(test)]
mod tests {

    use super::*;

    // ** WSEvent **

    #[test]
    fn test_wsevent_parse_1() {
        for et in WSEventType::iterator() {
            let et_s = et.to_string().to_lowercase();
            let str = format!("{}_a", et_s);
            let res = WSEventType::parse(&str);
            assert_eq!(res, None, "WSEventType={:?}", et);
        }
    }
    #[test]
    fn test_wsevent_parse_2() {
        for et in WSEventType::iterator() {
            let et_s = et.to_string().to_lowercase();
            let res = WSEventType::parse(&et_s);
            assert_eq!(res, Some(et.clone()), "WSEventType={:?}", et);
        }
    }
}
