pub const CD_PARSE_INT_ERROR: &str = "ParseIntError";
pub const MSG_PARSE_INT_ERROR: &str = "Failed conversion to i32 from";

// let val = parser::parse_i32("1a").map_err(|err| {
//     AppError::new(CD_PARSE_INT_ERROR, &format!("id: {err}")).set_status(400)
// })?;
pub fn parse_i32(val: &str) -> Result<i32, String> {
    val.parse::<i32>()
        .map_err(|e| format!("{MSG_PARSE_INT_ERROR} `{val}` - {}", e.to_string()))
}
