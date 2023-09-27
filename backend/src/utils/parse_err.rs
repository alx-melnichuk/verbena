pub const CD_PARSE_INT_ERROR: &str = "ParseIntError";
pub const MSG_PARSE_INT_ERROR: &str = "Failed conversion to i32 from";

// let msg = msg_parse_err("id", MSG_PARSE_INT_ERROR, &id_str, &e.to_string());
// AppError::new(CD_PARSE_INT_ERROR, &msg.to_string()).set_status(400)

pub fn msg_parse_err(field: &str, msg: &str, val: &str, err: &str) -> String {
    format!("{}: {} `{}` - {}", field, msg, val, err)
}
