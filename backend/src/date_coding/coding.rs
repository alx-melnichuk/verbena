use chrono::{DateTime, Datelike, NaiveDate, Timelike, Utc};

const BUFF: &str = "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";

pub const MSG_INVALID_LENGTH: &str = "Invalid data string length";
pub const MSG_INVALID_YEAR: &str = "Invalid year";
pub const MSG_INVALID_MILLISECOND: &str = "Invalid millisecond";
pub const MSG_INVALID_MONTH: &str = "Invalid month";
pub const MSG_INVALID_DAY: &str = "Invalid day";
pub const MSG_INVALID_HOUR: &str = "Invalid hour";
pub const MSG_INVALID_MINUTE: &str = "Invalid minute";
pub const MSG_INVALID_SECOND: &str = "Invalid second";

pub fn encode_eds(date_time: DateTime<Utc>) -> String {
    let buff_s = BUFF.to_string();
    let buff = buff_s.as_bytes();

    let year = format!("{:04}", date_time.year());
    let month = buff[date_time.month() as usize] as char;
    let day = buff[date_time.day() as usize] as char;

    let time = date_time.time();

    let hour = buff[time.hour() as usize] as char;
    let minute = buff[time.minute() as usize] as char;
    let second = buff[time.second() as usize] as char;
    let nanosec = format!("{:09}", time.nanosecond());

    let trio1 = format!("{}{}{}", &nanosec[0..1], &year[0..1], month);
    let trio2 = format!("{}{}{}", &nanosec[1..2], &year[1..2], day);
    let trio3 = format!("{}{}{}", &nanosec[2..3], &year[2..3], hour);
    let trio4 = format!("{}{}{}", &year[3..4], minute, second);

    format!("{}{}{}{}", trio1, trio2, trio3, trio4)
}

pub fn uncode_eds(value: String) -> Result<DateTime<Utc>, String> {
    let buff = BUFF.to_string();

    if value.len() != 12 {
        return Err(MSG_INVALID_LENGTH.to_string());
    }
    let mut chars = value.chars();

    let mill1 = chars.next().unwrap(); // [01] millisec1 +
    let year1 = chars.next().unwrap(); // [02] year1     +
    let month1 = chars.next().unwrap(); // [03] month    +

    let mill2 = chars.next().unwrap(); // [04] millisec2 +
    let year2 = chars.next().unwrap(); // [05] year2     +
    let day_1 = chars.next().unwrap(); // [06] day       +

    let mill3 = chars.next().unwrap(); // [07] millisec3 +
    let year3 = chars.next().unwrap(); // [08] year3     +
    let hour1 = chars.next().unwrap(); // [09] hour      +

    let year4 = chars.next().unwrap(); // [10] year4     +
    let minute = chars.next().unwrap(); // [11] minute   +
    let second = chars.next().unwrap(); // [12] second    !

    let year_s = format!("{}{}{}{}", year1, year2, year3, year4);
    let year = year_s
        .parse::<i32>()
        .map_err(|_| format!("{}: \"{}\"", MSG_INVALID_YEAR, year_s))?;

    let month_v = buff.chars().position(|c| c == month1).unwrap_or(0);
    if month_v < 1 || 12 < month_v {
        return Err(format!("{}: \"{}\"", MSG_INVALID_MONTH, month1));
    }
    let month = month_v.try_into().unwrap();

    let day_v = buff.chars().position(|c| c == day_1).unwrap_or(0);
    if day_v < 1 || 31 < day_v {
        return Err(format!("{}: \"{}\"", MSG_INVALID_DAY, day_1));
    }
    let day = day_v.try_into().unwrap();

    let hour_v = buff.chars().position(|c| c == hour1).unwrap_or(24);
    if hour_v > 23 {
        return Err(format!("{}: \"{}\"", MSG_INVALID_HOUR, hour1));
    }
    let hour = hour_v.try_into().unwrap();

    let minute_v = buff.chars().position(|c| c == minute).unwrap_or(60);
    if minute_v > 59 {
        return Err(format!("{}: \"{}\"", MSG_INVALID_MINUTE, minute));
    }
    let min = minute_v.try_into().unwrap();

    let second_v = buff.chars().position(|c| c == second).unwrap_or(60);
    if second_v > 59 {
        return Err(format!("{}: \"{}\"", MSG_INVALID_SECOND, second));
    }
    let sec = second_v.try_into().unwrap();

    let milli_s = format!("{}{}{}", mill1, mill2, mill3);
    let milli = milli_s
        .parse::<i32>()
        .map_err(|_| format!("{}: \"{}\"", MSG_INVALID_MILLISECOND, milli_s))?;
    let milli = milli as u32;

    let result = NaiveDate::from_ymd_opt(year, month, day)
        .unwrap()
        .and_hms_milli_opt(hour, min, sec, milli)
        .unwrap()
        .and_utc();

    Ok(result)
}

#[cfg(test)]
mod tests {

    use chrono::SecondsFormat;

    use super::*;

    #[test]
    fn test_encode_eds_valid_data() {
        let date_time = NaiveDate::from_ymd_opt(2015, 5, 15)
            .and_then(|d| d.and_hms_milli_opt(11, 12, 13, 678))
            .unwrap()
            .and_utc();

        let encode_date_time = encode_eds(date_time);

        assert_eq!("62570f81b5cd", encode_date_time);
    }
    #[test]
    fn test_uncode_eds_valid_data() {
        let value = "62570f81b5cd".to_string();
        let date_time: DateTime<Utc> = uncode_eds(value).unwrap();

        assert_eq!(2015, date_time.year());
        assert_eq!(5, date_time.month());
        assert_eq!(15, date_time.day());

        let time = date_time.time();
        assert_eq!(11, time.hour());
        assert_eq!(12, time.minute());
        assert_eq!(13, time.second());
        assert_eq!(678000000, time.nanosecond());
    }
    #[test]
    fn test_encode_eds_and_uncode_eds_valid_data() {
        let date_time = NaiveDate::from_ymd_opt(2015, 5, 15)
            .and_then(|d| d.and_hms_milli_opt(11, 12, 13, 678))
            .unwrap()
            .and_utc();

        let encode_date_time = encode_eds(date_time);

        let uncode_date_time: DateTime<Utc> = uncode_eds(encode_date_time).unwrap();

        let date_time_str = date_time.to_rfc3339_opts(SecondsFormat::Millis, true);
        let date_time_res = uncode_date_time.to_rfc3339_opts(SecondsFormat::Millis, true);

        assert_eq!(date_time_str, date_time_res);
    }
    #[test]
    fn test_uncode_eds_invalid_len() {
        let result = uncode_eds("62570f81b5cd0".to_string());

        assert!(result.clone().is_err());
        assert_eq!(result.unwrap_err(), MSG_INVALID_LENGTH);
    }
    /*#[test]
    fn test_uncode_eds_invalid_chair() {
        let result = uncode_eds("62570f81b5c_".to_string());

        assert!(result.clone().is_err());
        assert_eq!(result.unwrap_err(), format!("{} \"_\"", MSG_INVALID_CHARACTER));
    }*/
    #[test]
    fn test_uncode_eds_invalid_year1() {
        let result = uncode_eds("6Z570f81b5cd".to_string());

        assert!(result.clone().is_err());
        assert_eq!(result.unwrap_err(), format!("{}: \"Z015\"", MSG_INVALID_YEAR));
    }
    #[test]
    fn test_uncode_eds_invalid_year2() {
        let result = uncode_eds("6257Zf81b5cd".to_string());

        assert!(result.clone().is_err());
        assert_eq!(result.unwrap_err(), format!("{}: \"2Z15\"", MSG_INVALID_YEAR));
    }
    #[test]
    fn test_uncode_eds_invalid_year3() {
        let result = uncode_eds("62570f8Zb5cd".to_string());

        assert!(result.clone().is_err());
        assert_eq!(result.unwrap_err(), format!("{}: \"20Z5\"", MSG_INVALID_YEAR));
    }
    #[test]
    fn test_uncode_eds_invalid_year4() {
        let result = uncode_eds("62570f81bZcd".to_string());

        assert!(result.clone().is_err());
        assert_eq!(result.unwrap_err(), format!("{}: \"201Z\"", MSG_INVALID_YEAR));
    }
    #[test]
    fn test_uncode_eds_invalid_millisec1() {
        let result = uncode_eds("Z2570f81b5cd".to_string());

        assert!(result.clone().is_err());
        assert_eq!(result.unwrap_err(), format!("{}: \"Z78\"", MSG_INVALID_MILLISECOND));
    }
    #[test]
    fn test_uncode_eds_invalid_millisec2() {
        let result = uncode_eds("625Z0f81b5cd".to_string());

        assert!(result.clone().is_err());
        assert_eq!(result.unwrap_err(), format!("{}: \"6Z8\"", MSG_INVALID_MILLISECOND));
    }
    #[test]
    fn test_uncode_eds_invalid_millisec3() {
        let result = uncode_eds("62570fZ1b5cd".to_string());

        assert!(result.clone().is_err());
        assert_eq!(result.unwrap_err(), format!("{}: \"67Z\"", MSG_INVALID_MILLISECOND));
    }
    #[test]
    fn test_uncode_eds_invalid_month1() {
        let result = uncode_eds("62070f81b5cd".to_string());

        assert!(result.clone().is_err());
        assert_eq!(result.unwrap_err(), format!("{}: \"0\"", MSG_INVALID_MONTH));
    }
    #[test]
    fn test_uncode_eds_invalid_month2() {
        let result = uncode_eds("62d70f81b5cd".to_string());

        assert!(result.clone().is_err());
        assert_eq!(result.unwrap_err(), format!("{}: \"d\"", MSG_INVALID_MONTH));
    }
    #[test]
    fn test_uncode_eds_invalid_day1() {
        let result = uncode_eds("62570081b5cd".to_string());

        assert!(result.clone().is_err());
        assert_eq!(result.unwrap_err(), format!("{}: \"0\"", MSG_INVALID_DAY));
    }
    #[test]
    fn test_uncode_eds_invalid_day2() {
        let result = uncode_eds("62570w81b5cd".to_string());

        assert!(result.clone().is_err());
        assert_eq!(result.unwrap_err(), format!("{}: \"w\"", MSG_INVALID_DAY));
    }
    #[test]
    fn test_uncode_eds_invalid_hour1() {
        let result = uncode_eds("62570f81#5cd".to_string());

        assert!(result.clone().is_err());
        assert_eq!(result.unwrap_err(), format!("{}: \"#\"", MSG_INVALID_HOUR));
    }
    #[test]
    fn test_uncode_eds_invalid_hour2() {
        let result = uncode_eds("62570f81o5cd".to_string());

        assert!(result.clone().is_err());
        assert_eq!(result.unwrap_err(), format!("{}: \"o\"", MSG_INVALID_HOUR));
    }
    #[test]
    fn test_uncode_eds_invalid_minute1() {
        let result = uncode_eds("62570f81b5#d".to_string());

        assert!(result.clone().is_err());
        assert_eq!(result.unwrap_err(), format!("{}: \"#\"", MSG_INVALID_MINUTE));
    }
    #[test]
    fn test_uncode_eds_invalid_minute2() {
        let result = uncode_eds("62570f81b5Yd".to_string());

        assert!(result.clone().is_err());
        assert_eq!(result.unwrap_err(), format!("{}: \"Y\"", MSG_INVALID_MINUTE));
    }

    #[test]
    fn test_uncode_eds_invalid_second1() {
        let result = uncode_eds("62570f81b5c#".to_string());
        eprintln!("result: {:?}", &result);
        assert!(result.clone().is_err());
        assert_eq!(result.unwrap_err(), format!("{}: \"#\"", MSG_INVALID_SECOND));
    }
    #[test]
    fn test_uncode_eds_invalid_second2() {
        let result = uncode_eds("62570f81b5cY".to_string());
        eprintln!("result: {:?}", &result);
        assert!(result.clone().is_err());
        assert_eq!(result.unwrap_err(), format!("{}: \"Y\"", MSG_INVALID_SECOND));
    }

    // "62570f81b5cd"
}
