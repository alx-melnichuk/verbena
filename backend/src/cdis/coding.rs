use chrono::{DateTime, Datelike, NaiveDate, Timelike, Utc};

const BUFF: &str = "0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";

pub const MSG_INVALID_LENGTH: &str = "Invalid data string length";
pub const MSG_INVALID_YEAR: &str = "Invalid year";
pub const MSG_INVALID_MILLISECOND: &str = "Invalid millisecond";
pub const MSG_INVALID_MICROSECOND: &str = "Invalid microsecond";
pub const MSG_INVALID_NANOSECOND: &str = "Invalid nanosecond";
pub const MSG_INVALID_MONTH: &str = "Invalid month";
pub const MSG_INVALID_DAY: &str = "Invalid day";
pub const MSG_INVALID_HOUR: &str = "Invalid hour";
pub const MSG_INVALID_MINUTE: &str = "Invalid minute";
pub const MSG_INVALID_SECOND: &str = "Invalid second";

/// // Encode the date into a string.
/// // "accuracy" takes the following values:
/// // 0 - with milliseconds (10^3);
/// // 1 - with microseconds (10^6);
/// // 2 - with nanoseconds (10^9);
///
///
/// // Encode the date (with milliseconds, accuracy=0) into a string.
///
/// let date_time = NaiveDate::from_ymd_opt(2015, 5, 15)
///     .and_then(|d| d.and_hms_milli_opt(11, 12, 13, 456)).unwrap().and_utc();
///
/// let encode_date_time = encode(date_time, 0);
///
/// assert_eq!("42550f61b5cd", encode_date_time);
///
///
/// // Encode the date (with microseconds, accuracy=1) into a string.
///
/// let date_time = NaiveDate::from_ymd_opt(2015, 5, 15)
///     .and_then(|d| d.and_hms_micro_opt(11, 12, 13, 456789)).unwrap().and_utc();
///
/// let encode_date_time = encode(date_time, 1);
///
/// assert_eq!("42550f61b5cd987", encode_date_time);
///
///
/// // Encode the date (with nanoseconds, accuracy=2) into a string.
///
/// let date_time = NaiveDate::from_ymd_opt(2015, 5, 15)
///     .and_then(|d| d.and_hms_nano_opt(11, 12, 13, 456789123)).unwrap().and_utc();
///
/// let encode_date_time = encode(date_time, 2);
///
/// assert_eq!("42550f61b5cd987321", encode_date_time);
///

pub fn encode(date_time: DateTime<Utc>, accuracy: u8) -> String {
    let buff_s = BUFF.to_string();
    let buff = buff_s.as_bytes();
    let accuracy = if accuracy > 2 { 0 } else { accuracy };

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

    let trio5 = if accuracy > 0 {
        format!("{}{}{}", &nanosec[5..6], &nanosec[4..5], &nanosec[3..4])
    } else {
        "".to_string()
    };
    let trio6 = if accuracy > 1 {
        format!("{}{}{}", &nanosec[8..9], &nanosec[7..8], &nanosec[6..7])
    } else {
        "".to_string()
    };

    format!("{}{}{}{}{}{}", trio1, trio2, trio3, trio4, trio5, trio6)
}

///
/// // Decode a string into a date.
/// // "accuracy" takes the following values:
/// // 0 - in milliseconds (10^3);
/// // 1 - in microseconds (10^6);
/// // 2 - in nanoseconds (10^9);
///
///
/// // Decode a string (in milliseconds, accuracy=0) into a date.
///
/// let date_time: DateTime<Utc> = decode("42550f61b5cd".to_string(), 0).unwrap();
///
/// assert_eq!(2015, date_time.year());
/// assert_eq!(5, date_time.month());
/// assert_eq!(15, date_time.day());
///
/// let time = date_time.time();
/// assert_eq!(11, time.hour());
/// assert_eq!(12, time.minute());
/// assert_eq!(13, time.second());
/// assert_eq!(456000000, time.nanosecond());
///
///
/// // Decode a string (in microseconds, accuracy=1) into a date.
///
/// let date_time: DateTime<Utc> = decode("42550f61b5cd987", 1).unwrap();
///
/// assert_eq!(2015, date_time.year());
/// assert_eq!(5, date_time.month());
/// assert_eq!(15, date_time.day());
///
/// let time = date_time.time();
/// assert_eq!(11, time.hour());
/// assert_eq!(12, time.minute());
/// assert_eq!(13, time.second());
/// assert_eq!(456789000, time.nanosecond());
///
///
/// // Decode a string (in nanoseconds, accuracy=2) into a date.
///
/// let date_time: DateTime<Utc> = decode("42550f61b5cd987321", 2).unwrap();
///
/// assert_eq!(2015, date_time.year());
/// assert_eq!(5, date_time.month());
/// assert_eq!(15, date_time.day());
///
/// let time = date_time.time();
/// assert_eq!(11, time.hour());
/// assert_eq!(12, time.minute());
/// assert_eq!(13, time.second());
/// assert_eq!(456789123, time.nanosecond());
///

pub fn decode(value: &str, accuracy: u8) -> Result<DateTime<Utc>, String> {
    let accuracy = if accuracy > 2 { 0 } else { accuracy };
    let max_len: usize = (12 + 3 * accuracy).into();

    if value.len() != max_len {
        return Err(MSG_INVALID_LENGTH.to_string());
    }
    let month1 = value[02..03].as_bytes()[0] as char;
    let day_1 = value[05..06].as_bytes()[0] as char;
    let hour1 = value[08..09].as_bytes()[0] as char;
    let minute = value[10..11].as_bytes()[0] as char;
    let second = value[11..12].as_bytes()[0] as char;

    let nano123_s = format!("{}{}{}", &value[0..1], &value[3..4], &value[6..7]);

    let nano456_s = if accuracy > 0 {
        format!("{}{}{}", &value[14..15], &value[13..14], &value[12..13])
    } else {
        "".to_string()
    };

    let nano789_s = if accuracy > 1 {
        format!("{}{}{}", &value[17..18], &value[16..17], &value[15..16])
    } else {
        "".to_string()
    };

    let year_s = format!("{}{}{}{}", &value[1..2], &value[4..5], &value[7..8], &value[9..10]);

    let year = year_s
        .parse::<i32>()
        .map_err(|_| format!("{}: \"{}\"", MSG_INVALID_YEAR, year_s))?;

    let nano123 = nano123_s
        .parse::<i32>()
        .map_err(|_| format!("{}: \"{}\"", MSG_INVALID_MILLISECOND, nano123_s))?;
    let mut nano = nano123 * 1000000;

    if accuracy > 0 {
        let nano456_v = nano456_s
            .parse::<i32>()
            .map_err(|_| format!("{}: \"{}\"", MSG_INVALID_MICROSECOND, nano456_s))?;
        nano += nano456_v * 1000;
    }

    if accuracy > 1 {
        let nano789_v = nano789_s
            .parse::<i32>()
            .map_err(|_| format!("{}: \"{}\"", MSG_INVALID_NANOSECOND, nano789_s))?;
        nano += nano789_v;
    }

    let nano = nano as u32;

    let mut month_v = 0;
    let mut day_v = 0;
    let mut hour_v = 24;
    let mut minute_v = 60;
    let mut second_v = 60;

    let buff = BUFF.as_bytes();
    let buff_len = buff.len();
    let mut idx = 0;
    while idx < buff_len {
        let char = buff[idx..(idx + 1)][0] as char;

        if 0 == month_v && char == month1 {
            month_v = idx;
        }
        if 0 == day_v && char == day_1 {
            day_v = idx;
        }
        if 24 == hour_v && char == hour1 {
            hour_v = idx;
        }
        if 60 == minute_v && char == minute {
            minute_v = idx;
        }
        if 60 == second_v && char == second {
            second_v = idx;
        }

        if month_v != 0 && day_v != 0 && hour_v != 24 && minute_v != 60 && second_v != 60 {
            break;
        }
        idx += 1;
    }

    if month_v < 1 || 12 < month_v {
        return Err(format!("{}: \"{}\"", MSG_INVALID_MONTH, month1));
    }
    let month = month_v.try_into().unwrap();

    if day_v < 1 || 31 < day_v {
        return Err(format!("{}: \"{}\"", MSG_INVALID_DAY, day_1));
    }
    let day = day_v.try_into().unwrap();

    if hour_v > 23 {
        return Err(format!("{}: \"{}\"", MSG_INVALID_HOUR, hour1));
    }
    let hour = hour_v.try_into().unwrap();

    if minute_v > 59 {
        return Err(format!("{}: \"{}\"", MSG_INVALID_MINUTE, minute));
    }
    let min = minute_v.try_into().unwrap();

    if second_v > 59 {
        return Err(format!("{}: \"{}\"", MSG_INVALID_SECOND, second));
    }
    let sec = second_v.try_into().unwrap();

    let result = NaiveDate::from_ymd_opt(year, month, day)
        .unwrap()
        .and_hms_nano_opt(hour, min, sec, nano)
        .unwrap()
        .and_utc();
    Ok(result)
}

#[cfg(test)]
mod tests {

    use super::*;

    // ** Millisecond-accuracy encoding and decoding. **

    #[test]
    fn test_encode_valid_data_millisec() {
        let date_time = NaiveDate::from_ymd_opt(2015, 5, 15)
            .and_then(|d| d.and_hms_milli_opt(11, 12, 13, 456))
            .unwrap()
            .and_utc();

        let encode_date_time = encode(date_time, 0);

        assert_eq!("42550f61b5cd", encode_date_time);
    }
    #[test]
    fn test_decode_valid_data_millisec() {
        let date_time: DateTime<Utc> = decode("42550f61b5cd", 0).unwrap();

        assert_eq!(2015, date_time.year());
        assert_eq!(5, date_time.month());
        assert_eq!(15, date_time.day());

        let time = date_time.time();
        assert_eq!(11, time.hour());
        assert_eq!(12, time.minute());
        assert_eq!(13, time.second());
        assert_eq!(456000000, time.nanosecond());
    }
    #[test]
    fn test_encode_and_decode_valid_data_millisec() {
        let date_time = NaiveDate::from_ymd_opt(2015, 5, 15)
            .and_then(|d| d.and_hms_milli_opt(11, 12, 13, 678))
            .unwrap()
            .and_utc();

        let encode_date_time = encode(date_time, 0);

        let uncode_date_time: DateTime<Utc> = decode(&encode_date_time, 0).unwrap();

        let date_format = "%Y-%m-%d %H:%M:%S%.9f %z";
        let date_time_str = date_time.format(date_format).to_string();
        let date_time_res = uncode_date_time.format(date_format).to_string();

        assert_eq!(date_time_str, date_time_res);
    }

    // ** Microsecond-accuracy encoding and decoding. **

    #[test]
    fn test_encode_valid_data_microsec() {
        let date_time = NaiveDate::from_ymd_opt(2015, 5, 15)
            .and_then(|d| d.and_hms_micro_opt(11, 12, 13, 456789))
            .unwrap()
            .and_utc();

        let encode_date_time = encode(date_time, 1);

        assert_eq!("42550f61b5cd987", encode_date_time);
    }
    #[test]
    fn test_decode_valid_data_microsec() {
        let date_time: DateTime<Utc> = decode("42550f61b5cd987", 1).unwrap();

        assert_eq!(2015, date_time.year());
        assert_eq!(5, date_time.month());
        assert_eq!(15, date_time.day());

        let time = date_time.time();
        assert_eq!(11, time.hour());
        assert_eq!(12, time.minute());
        assert_eq!(13, time.second());
        assert_eq!(456789000, time.nanosecond());
    }
    #[test]
    fn test_encode_and_decode_valid_data_microsec() {
        let date_time = NaiveDate::from_ymd_opt(2015, 5, 15)
            .and_then(|d| d.and_hms_micro_opt(11, 12, 13, 456789))
            .unwrap()
            .and_utc();

        let encode_date_time = encode(date_time, 1);

        let uncode_date_time: DateTime<Utc> = decode(&encode_date_time, 1).unwrap();

        let date_format = "%Y-%m-%d %H:%M:%S%.9f %z";
        let date_time_str = date_time.format(date_format).to_string();
        let date_time_res = uncode_date_time.format(date_format).to_string();

        assert_eq!(date_time_str, date_time_res);
    }

    // ** Nanosecond-accuracy encoding and decoding. **

    #[test]
    fn test_encode_valid_data_nanosec() {
        let date_time = NaiveDate::from_ymd_opt(2015, 5, 15)
            .and_then(|d| d.and_hms_nano_opt(11, 12, 13, 456789123))
            .unwrap()
            .and_utc();

        let encode_date_time = encode(date_time, 2);

        assert_eq!("42550f61b5cd987321", encode_date_time);
    }
    #[test]
    fn test_decode_valid_data_nanosec() {
        let date_time: DateTime<Utc> = decode("42550f61b5cd987321", 2).unwrap();

        assert_eq!(2015, date_time.year());
        assert_eq!(5, date_time.month());
        assert_eq!(15, date_time.day());

        let time = date_time.time();
        assert_eq!(11, time.hour());
        assert_eq!(12, time.minute());
        assert_eq!(13, time.second());
        assert_eq!(456789123, time.nanosecond());
    }
    #[test]
    fn test_encode_and_decode_valid_data_nanosec() {
        let date_time = NaiveDate::from_ymd_opt(2015, 5, 15)
            .and_then(|d| d.and_hms_nano_opt(11, 12, 13, 456789123))
            .unwrap()
            .and_utc();

        let encode_date_time = encode(date_time, 2);

        let uncode_date_time: DateTime<Utc> = decode(&encode_date_time, 2).unwrap();

        let date_format = "%Y-%m-%d %H:%M:%S%.9f %z";
        let date_time_str = date_time.format(date_format).to_string();
        let date_time_res = uncode_date_time.format(date_format).to_string();

        assert_eq!(date_time_str, date_time_res);
    }

    // ** Decoding with invalid length. **

    #[test]
    fn test_decode_invalid_len_millisec() {
        let result = decode("62570f81b5cd0", 0);

        assert!(result.clone().is_err());
        assert_eq!(result.unwrap_err(), MSG_INVALID_LENGTH);
    }
    #[test]
    fn test_decode_invalid_len_microsec() {
        let result = decode("42550f61b5cd9870", 1);

        assert!(result.clone().is_err());
        assert_eq!(result.unwrap_err(), MSG_INVALID_LENGTH);
    }
    #[test]
    fn test_decode_invalid_len_nanosec() {
        let result = decode("42550f61b5cd9873210", 2);

        assert!(result.clone().is_err());
        assert_eq!(result.unwrap_err(), MSG_INVALID_LENGTH);
    }

    // ** Decoding with invalid year. **

    #[test]
    fn test_decode_invalid_year1() {
        let result = decode("6Z570f81b5cd", 0);

        assert!(result.clone().is_err());
        assert_eq!(result.unwrap_err(), format!("{}: \"Z015\"", MSG_INVALID_YEAR));
    }
    #[test]
    fn test_decode_invalid_year2() {
        let result = decode("6257Zf81b5cd", 0);

        assert!(result.clone().is_err());
        assert_eq!(result.unwrap_err(), format!("{}: \"2Z15\"", MSG_INVALID_YEAR));
    }
    #[test]
    fn test_decode_invalid_year3() {
        let result = decode("62570f8Zb5cd", 0);

        assert!(result.clone().is_err());
        assert_eq!(result.unwrap_err(), format!("{}: \"20Z5\"", MSG_INVALID_YEAR));
    }
    #[test]
    fn test_decode_invalid_year4() {
        let result = decode("62570f81bZcd", 0);

        assert!(result.clone().is_err());
        assert_eq!(result.unwrap_err(), format!("{}: \"201Z\"", MSG_INVALID_YEAR));
    }

    // ** Decoding with invalid month. **

    #[test]
    fn test_decode_invalid_month1() {
        let result = decode("62070f81b5cd", 0);

        assert!(result.clone().is_err());
        assert_eq!(result.unwrap_err(), format!("{}: \"0\"", MSG_INVALID_MONTH));
    }
    #[test]
    fn test_decode_invalid_month2() {
        let result = decode("62d70f81b5cd", 0);

        assert!(result.clone().is_err());
        assert_eq!(result.unwrap_err(), format!("{}: \"d\"", MSG_INVALID_MONTH));
    }

    // ** Decoding with invalid day. **

    #[test]
    fn test_decode_invalid_day1() {
        let result = decode("62570081b5cd", 0);

        assert!(result.clone().is_err());
        assert_eq!(result.unwrap_err(), format!("{}: \"0\"", MSG_INVALID_DAY));
    }
    #[test]
    fn test_decode_invalid_day2() {
        let result = decode("62570w81b5cd", 0);

        assert!(result.clone().is_err());
        assert_eq!(result.unwrap_err(), format!("{}: \"w\"", MSG_INVALID_DAY));
    }

    // ** Decoding with invalid hour. **

    #[test]
    fn test_decode_invalid_hour1() {
        let result = decode("62570f81#5cd", 0);

        assert!(result.clone().is_err());
        assert_eq!(result.unwrap_err(), format!("{}: \"#\"", MSG_INVALID_HOUR));
    }
    #[test]
    fn test_decode_invalid_hour2() {
        let result = decode("62570f81o5cd", 0);

        assert!(result.clone().is_err());
        assert_eq!(result.unwrap_err(), format!("{}: \"o\"", MSG_INVALID_HOUR));
    }

    // ** Decoding with invalid minute. **

    #[test]
    fn test_decode_invalid_minute1() {
        let result = decode("62570f81b5#d", 0);

        assert!(result.clone().is_err());
        assert_eq!(result.unwrap_err(), format!("{}: \"#\"", MSG_INVALID_MINUTE));
    }
    #[test]
    fn test_decode_invalid_minute2() {
        let result = decode("62570f81b5Yd", 0);

        assert!(result.clone().is_err());
        assert_eq!(result.unwrap_err(), format!("{}: \"Y\"", MSG_INVALID_MINUTE));
    }

    // ** Decoding with invalid second. **

    #[test]
    fn test_decode_invalid_second1() {
        let result = decode("62570f81b5c#", 0);
        eprintln!("result: {:?}", &result);
        assert!(result.clone().is_err());
        assert_eq!(result.unwrap_err(), format!("{}: \"#\"", MSG_INVALID_SECOND));
    }
    #[test]
    fn test_decode_invalid_second2() {
        let result = decode("62570f81b5cY", 0);
        eprintln!("result: {:?}", &result);
        assert!(result.clone().is_err());
        assert_eq!(result.unwrap_err(), format!("{}: \"Y\"", MSG_INVALID_SECOND));
    }

    // ** Decoding with invalid millisecond. **

    #[test]
    fn test_decode_invalid_millisec1() {
        let result = decode("Z2570f81b5cd", 0);

        assert!(result.clone().is_err());
        assert_eq!(result.unwrap_err(), format!("{}: \"Z78\"", MSG_INVALID_MILLISECOND));
    }
    #[test]
    fn test_decode_invalid_millisec2() {
        let result = decode("625Z0f81b5cd", 0);

        assert!(result.clone().is_err());
        assert_eq!(result.unwrap_err(), format!("{}: \"6Z8\"", MSG_INVALID_MILLISECOND));
    }
    #[test]
    fn test_decode_invalid_millisec3() {
        let result = decode("62570fZ1b5cd", 0);

        assert!(result.clone().is_err());
        assert_eq!(result.unwrap_err(), format!("{}: \"67Z\"", MSG_INVALID_MILLISECOND));
    }

    // ** Decoding with invalid millisecond. **

    #[test]
    fn test_decode_invalid_microsec4() {
        let result = decode("42550f61b5cd98Z", 1);

        assert!(result.clone().is_err());
        assert_eq!(result.unwrap_err(), format!("{}: \"Z89\"", MSG_INVALID_MICROSECOND));
    }
    #[test]
    fn test_decode_invalid_microsec5() {
        let result = decode("42550f61b5cd9Z7", 1);

        assert!(result.clone().is_err());
        assert_eq!(result.unwrap_err(), format!("{}: \"7Z9\"", MSG_INVALID_MICROSECOND));
    }
    #[test]
    fn test_decode_invalid_microsec6() {
        let result = decode("42550f61b5cdZ87", 1);

        assert!(result.clone().is_err());
        assert_eq!(result.unwrap_err(), format!("{}: \"78Z\"", MSG_INVALID_MICROSECOND));
    }

    // ** Decoding with invalid nanosecond. ** "42550f61b5cd987321"

    #[test]
    fn test_decode_invalid_nanosec7() {
        let result = decode("42550f61b5cd98732Z", 2);

        assert!(result.clone().is_err());
        assert_eq!(result.unwrap_err(), format!("{}: \"Z23\"", MSG_INVALID_NANOSECOND));
    }
    #[test]
    fn test_decode_invalid_nanosec8() {
        let result = decode("42550f61b5cd9873Z1", 2);

        assert!(result.clone().is_err());
        assert_eq!(result.unwrap_err(), format!("{}: \"1Z3\"", MSG_INVALID_NANOSECOND));
    }
    #[test]
    fn test_decode_invalid_nanosec9() {
        let result = decode("42550f61b5cd987Z21", 2);

        assert!(result.clone().is_err());
        assert_eq!(result.unwrap_err(), format!("{}: \"12Z\"", MSG_INVALID_NANOSECOND));
    }
}
