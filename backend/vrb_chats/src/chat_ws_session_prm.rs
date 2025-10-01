use actix_broker::BrokerMsg;
use log::debug;
use serde_json::to_string;

use crate::{chat_event_ws::{EWSType, ErrEWS, EventWS, PrmBoolEWS, PrmIntEWS, PrmStrEWS}, chat_message::SendMessage, chat_ws_tools};

#[derive(Debug, Clone)]
pub struct ChatWsSessionPrmInfo {
    room_id: i32,
    is_blocked: bool,
    is_owner: bool,
}

impl ChatWsSessionPrmInfo {
    pub fn new(room_id: i32, is_blocked: bool, is_owner: bool) -> ChatWsSessionPrmInfo {
        ChatWsSessionPrmInfo { room_id, is_blocked, is_owner }
    }
}

pub trait ChatWsSessionPrm {

    fn get_prm_info(&self) -> ChatWsSessionPrmInfo;

    fn prm_issue_system_async<M: BrokerMsg>(&self, msg: M);

    fn handle_event_ews_type(&self, event: EventWS) -> Result<(), ErrEWS> {
        match event.ews_type() {
            EWSType::PrmBool => {
                // {"prmBool": "paramB", "valBool": true }
                let prm_bool = event.get_string("prmBool").unwrap_or("".to_owned());
                let opt_val_bool = event.get_bool("valBool");
                self.handle_ews_prm_bool(&prm_bool, opt_val_bool)?;
                Ok(())
            }
            EWSType::PrmInt => {
                // {"prmInt": "paramI", "valInt": 10 }
                let prm_int = event.get_string("prmInt").unwrap_or("".to_owned());
                let opt_val_int = event.get_i32("valInt");
                self.handle_ews_prm_int(&prm_int, opt_val_int)?;
                Ok(())
            }
            EWSType::PrmStr => {
                // {"prmStr": "paramS", "valStr": "text" }
                let prm_str = event.get_string("prmStr").unwrap_or("".to_owned());
                let opt_val_str = event.get_string("valStr");
                self.handle_ews_prm_str(&prm_str, opt_val_str)?;
                Ok(())
            }
            _ => {
                Ok(())
            }
        }
    }

    fn handle_ews_prm_bool(&self, prm_bool: &str, opt_val_bool: Option<bool>) -> Result<(), ErrEWS> {
        debug!("handle_ews_prm_bool() prm_bool: {}, opt_val_bool: {:?}", prm_bool, opt_val_bool);
        // Check if this field is not empty
        chat_ws_tools::check_is_not_empty(prm_bool, "prmBool")?;
        // Check if this field is required
        chat_ws_tools::check_is_required(opt_val_bool, "valBool")?;
        let val_bool = opt_val_bool.unwrap();
        let prm_info = self.get_prm_info();
        let room_id = prm_info.room_id;
        // Check if there is an joined room
        chat_ws_tools::check_is_joined_room(room_id)?;
        // Check if there is a block on sending messages
        chat_ws_tools::check_is_blocked(prm_info.is_blocked)?;

        let prm_bool = prm_bool.to_owned();
        let is_owner: Option<bool> = if prm_info.is_owner { Some(true) } else { None };
        #[rustfmt::skip]
        let prm_int_str = to_string(&PrmBoolEWS { prm_bool, val_bool, is_owner }).unwrap();
        // issue_async comes from having the `BrokerIssue` trait in scope.
        self.prm_issue_system_async(SendMessage(room_id, prm_int_str));
        Ok(())
    }

    fn handle_ews_prm_int(&self, prm_int: &str, opt_val_int: Option<i32>) -> Result<(), ErrEWS> {
        debug!("handle_ews_prm_int() prm_int: {}, opt_val_int: {:?}", prm_int, opt_val_int);
        // Check if this field is not empty
        chat_ws_tools::check_is_not_empty(prm_int, "prmInt")?;
        // Check if this field is required
        chat_ws_tools::check_is_required(opt_val_int, "valInt")?;
        let val_int = opt_val_int.unwrap();
        let prm_info = self.get_prm_info();
        let room_id = prm_info.room_id;
        // Check if there is an joined room
        chat_ws_tools::check_is_joined_room(room_id)?;
        // Check if there is a block on sending messages
        chat_ws_tools::check_is_blocked(prm_info.is_blocked)?;

        let prm_int = prm_int.to_owned();
        let is_owner: Option<bool> = if prm_info.is_owner { Some(true) } else { None };
        #[rustfmt::skip]
        let prm_int_str = to_string(&PrmIntEWS { prm_int, val_int, is_owner }).unwrap();
        // issue_async comes from having the `BrokerIssue` trait in scope.
        self.prm_issue_system_async(SendMessage(room_id, prm_int_str));
        Ok(())
    }

    fn handle_ews_prm_str(&self, prm_str: &str, opt_val_str: Option<String>) -> Result<(), ErrEWS> {
        debug!("handle_ews_prm_str() prm_str: {}, opt_val_str: {:?}", prm_str, opt_val_str);
        // Check if this field is not empty
        chat_ws_tools::check_is_not_empty(prm_str, "prmStr")?;
        // Check if this field is required
        chat_ws_tools::check_is_required(opt_val_str.clone(), "valStr")?;
        let val_str = opt_val_str.unwrap();
        let prm_info = self.get_prm_info();
        let room_id = prm_info.room_id;
        // Check if there is an joined room
        chat_ws_tools::check_is_joined_room(room_id)?;
        // Check if there is a block on sending messages
        chat_ws_tools::check_is_blocked(prm_info.is_blocked)?;

        let prm_str = prm_str.to_owned();
        let is_owner: Option<bool> = if prm_info.is_owner { Some(true) } else { None };
        #[rustfmt::skip]
        let prm_int_str = to_string(&PrmStrEWS { prm_str, val_str, is_owner }).unwrap();
        // issue_async comes from having the `BrokerIssue` trait in scope.
        self.prm_issue_system_async(SendMessage(room_id, prm_int_str));
        Ok(())
    }
}