use std::slice::Iter;
use std::{collections::HashMap, fmt};

use serde::{Deserialize, Serialize};
use serde_json;

// ** Types of events that are transmitted over a websocket. **

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum EWSType {
    Block,
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
        static LIST: [EWSType; 10] = [
            EWSType::Block,
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
    params: HashMap<String, String>,
}

impl EventWS {
    pub fn new(et: EWSType, params_opt: Option<HashMap<String, String>>) -> Self {
        EventWS {
            et,
            params: params_opt.unwrap_or(HashMap::new()),
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

        let res_ews_type = EWSType::parse(first_tag);
        if let None = res_ews_type {
            return Err(format!("unknown command: {:?}", event).to_string());
        }
        let ews_type = res_ews_type.unwrap();

        let mut params: HashMap<String, String> = HashMap::new();
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
            params.insert(key.clone(), value);
        }
        Ok(EventWS::new(ews_type, Some(params)))
    }

    pub fn ews_type(&self) -> EWSType {
        self.et.clone()
    }

    pub fn get(&self, name: &str) -> Option<String> {
        self.params.get(name).map(|s| s.clone())
    }
}

// ** **

// ** Block clients in a room by name. **
#[derive(Serialize, Deserialize, Clone)]
pub struct BlockEWS {
    pub block: String,
    pub count: u32,
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
    pub join: String,
    pub member: String,
    pub count: usize,
}

// ** Leave the client from the chat room. **
#[derive(Serialize, Deserialize, Clone)]
pub struct LeaveEWS {
    pub leave: String,
    pub member: String,
    pub count: usize,
}

// ** Send a text message to all clients in the room. **
#[derive(Serialize, Deserialize, Clone)]
pub struct MsgEWS {
    pub msg: String,
    pub member: String,
    pub date: String,
}

// ** Send a delete message to all chat members. **
#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MsgCutEWS {
    pub msg_cut: String,
    pub member: String,
    pub date: String,
}

// ** Send a correction to the message to everyone in the chat. **
#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MsgPutEWS {
    pub msg_put: String,
    pub member: String,
    pub date: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct NameEWS {
    pub name: String,
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
