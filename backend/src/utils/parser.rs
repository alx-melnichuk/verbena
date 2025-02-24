pub const MSG_PARSE_INT_ERROR: &str = "Failed conversion to `i32` from";
pub const MSG_PARSE_BOOL_ERROR: &str = "Failed conversion to `bool` from";

pub fn parse_i32(val: &str) -> Result<i32, String> {
    val.parse::<i32>().map_err(|e| format!("{} ({})", e.to_string(), val))
}
pub fn parse_bool(val: &str) -> Result<bool, String> {
    val.parse::<bool>().map_err(|e| format!("{} ({})", e.to_string(), val))
}
