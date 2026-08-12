use actix_web::http::StatusCode;
use vrb_common::err;

use crate::chat_event_ws::ErrEWS;

#[rustfmt::skip]
pub fn get_err400(message: &str) -> ErrEWS {
    ErrEWS { status: StatusCode::BAD_REQUEST.as_u16(), message: message.to_owned() } // 400
}
#[rustfmt::skip]
pub fn get_err401(message: &str) -> ErrEWS {
    ErrEWS { status: StatusCode::UNAUTHORIZED.as_u16(), message: message.to_owned() } // 401
}
#[rustfmt::skip]
pub fn get_err403(message: &str) -> ErrEWS {
    ErrEWS { status: StatusCode::FORBIDDEN.as_u16(), message: message.to_owned() } // 403
}
#[rustfmt::skip]
pub fn get_err404(message: &str) -> ErrEWS {
    ErrEWS { status: StatusCode::NOT_FOUND.as_u16(), message: message.to_owned() } // 404
}
#[rustfmt::skip]
pub fn get_err406(message: &str) -> ErrEWS {
    ErrEWS { status: StatusCode::NOT_ACCEPTABLE.as_u16(), message: message.to_owned() } // 406
}
#[rustfmt::skip]
pub fn get_err409(message: &str) -> ErrEWS {
    ErrEWS { status: StatusCode::CONFLICT.as_u16(), message: message.to_owned() } // 409
}
#[rustfmt::skip]
pub fn get_err500(message: &str) -> ErrEWS {
    ErrEWS { status: StatusCode::INTERNAL_SERVER_ERROR.as_u16(), message: message.to_owned() } // 500
}

// Check if this field is required
pub fn check_is_not_empty(value: &str, name: &str) -> Result<(), ErrEWS> {
    if value.len() == 0 {
        Err(get_err400(&format!("{}; name: '{}'", err::MSG_PARAMETER_NOT_DEFINED, name)))
    } else {
        Ok(())
    }
}

pub fn check_is_required<T>(value: Option<T>, name: &str) -> Result<(), ErrEWS> {
    if value.is_none() {
        Err(get_err400(&format!("{}; name: '{}'", err::MSG_PARAMETER_NOT_DEFINED, name)))
    } else {
        Ok(())
    }
}

pub fn check_is_greater_than(value: i32, limit: i32, name: &str) -> Result<(), ErrEWS> {
    if value <= limit {
        Err(get_err400(&format!("{}; name: '{}'", err::MSG_PARAMETER_NOT_DEFINED, name)))
    } else {
        Ok(())
    }
}
// Check if there is an joined room
pub fn check_is_joined_room(room_id: i32) -> Result<(), ErrEWS> {
    if room_id <= i32::default() {
        Err(get_err406(err::MSG_THERE_WAS_NO_JOIN))
    } else {
        Ok(())
    }
}
// Check if there is a block on sending messages
pub fn check_is_blocked(is_blocked: bool) -> Result<(), ErrEWS> {
    if is_blocked {
        Err(get_err403(err::MSG_BLOCK_ON_SEND_MESSAGES))
    } else {
        Ok(())
    }
}
// Check if the user is the owner of the stream.
pub fn check_is_owner_room(is_owner: bool) -> Result<(), ErrEWS> {
    if !is_owner {
        Err(get_err403(err::MSG_STREAM_OWNER_RIGHTS_MISSING))
    } else {
        Ok(())
    }
}
