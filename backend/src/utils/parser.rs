pub const MSG_PARSE_INT_ERROR: &str = "Failed conversion to `i32` from";
pub const MSG_PARSE_BOOL_ERROR: &str = "Failed conversion to `bool` from";

// let val = parser::parse_i32("1a").map_err(|err| {
//     AppError::new(err::CD_PARSE_ERROR, &format!("id: {}", err)).set_status(400)
// })?;
pub fn parse_i32(val: &str) -> Result<i32, String> {
    val.parse::<i32>()
        .map_err(|e| format!("{} `{}` - {}", MSG_PARSE_INT_ERROR, val, e.to_string()))
}
pub fn parse_bool(val: &str) -> Result<bool, String> {
    val.parse::<bool>()
        .map_err(|e| format!("{} `{}` - {}", MSG_PARSE_BOOL_ERROR, val, e.to_string()))
}
