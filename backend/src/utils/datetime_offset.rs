use chrono::{DateTime, Duration, Local, Offset, Utc};

/// Convert date and time with new time offset value relative to Utc.
pub fn get_datetime_for_offset(local_now: DateTime<Local>, client_mins_utc: i32) -> DateTime<Utc> {
    // Get the time offset for the current locale.
    let local_offset = local_now.offset().fix();
    // Get the number of minutes to add to convert from UTC to the local time.
    let local_mins = -local_offset.local_minus_utc() / 60;
    // Get the difference between the time offset for the current locale and the specified input parameter.
    let delta = client_mins_utc - local_mins;
    // Change the current time to the resulting difference.
    let client_now = local_now - Duration::minutes(i64::from(delta));

    client_now.to_utc()
}

#[cfg(test)]
mod tests {
    use chrono::{Datelike, Local, Offset, TimeZone, Timelike};

    use super::get_datetime_for_offset;

    // ** get_datetime_for_offset **
    #[test]
    fn test_get_datetime_for_offset_with_offset_value() {
        let date = Local.with_ymd_and_hms(2020, 1, 1, 18, 0, 0).unwrap();
        let date_utc = date.to_utc();
        // "2020-01-01T16:00:00Z"
        let offset = date.offset().fix().local_minus_utc() / 60;
        let delta = -offset; // -120

        let res_utc = get_datetime_for_offset(date, delta);
        // "2020-01-01T16:00:00Z"
        assert_eq!(date_utc.year(), res_utc.year());
        assert_eq!(date_utc.month(), res_utc.month());
        assert_eq!(date_utc.day(), res_utc.day());
        assert_eq!(date_utc.hour(), res_utc.hour());
        assert_eq!(date_utc.minute(), res_utc.minute());
        assert_eq!(date_utc.second(), res_utc.second());
    }
    #[test]
    fn test_get_datetime_for_offset_with_offset_zero() {
        let date = Local.with_ymd_and_hms(2020, 1, 1, 18, 0, 0).unwrap();
        let date_utc = date.to_utc();
        // "2020-01-01T16:00:00Z"
        let offset = date.offset().fix().local_minus_utc() / 60;
        let delta_hours = offset / 60;
        // let delta_minutes = offset - delta_hours * 60;

        let res_utc = get_datetime_for_offset(date, 0);
        // "2020-01-01T16:00:00Z"
        assert_eq!(date_utc.year(), res_utc.year());
        assert_eq!(date_utc.month(), res_utc.month());
        assert_eq!(date_utc.day(), res_utc.day());
        assert_eq!(date_utc.hour() as i32 - delta_hours, res_utc.hour() as i32);
        // assert_eq!(date_utc.minute() as i32 - delta_minutes, res_utc.minute() as i32);
        assert_eq!(date_utc.second(), res_utc.second());
    }
    #[test]
    fn test_get_datetime_for_offset_with_offset_back_30_min() {
        let date = Local.with_ymd_and_hms(2020, 1, 1, 18, 0, 0).unwrap();
        let date_utc = date.to_utc();
        // "2020-01-01T16:00:00Z"

        let offset = date.offset().fix().local_minus_utc() / 60;
        let delta = -(offset - 30); // -90

        let res_utc = get_datetime_for_offset(date, delta);
        // "2020-01-01T15:30:00Z"
        assert_eq!(date_utc.year(), res_utc.year());
        assert_eq!(date_utc.month(), res_utc.month());
        assert_eq!(date_utc.day(), res_utc.day());
        assert_eq!(date_utc.hour() - 1, res_utc.hour());
        assert_eq!(date_utc.minute() + 30, res_utc.minute());
        assert_eq!(date_utc.second(), res_utc.second());
    }
    #[test]
    fn test_get_datetime_for_offset_with_offset_forward_30_min() {
        let date = Local.with_ymd_and_hms(2020, 1, 1, 18, 0, 0).unwrap();
        let date_utc = date.to_utc();
        // "2020-01-01T16:00:00Z"
        let offset = date.offset().fix().local_minus_utc() / 60;
        let delta = -(offset + 30); // -150

        let res_utc = get_datetime_for_offset(date, delta);
        // "2020-01-01T16:30:00Z"
        assert_eq!(date_utc.year(), res_utc.year());
        assert_eq!(date_utc.month(), res_utc.month());
        assert_eq!(date_utc.day(), res_utc.day());
        assert_eq!(date_utc.hour(), res_utc.hour());
        assert_eq!(date_utc.minute() + 30, res_utc.minute());
        assert_eq!(date_utc.second(), res_utc.second());
    }

    #[test]
    fn test_get_datetime_for_offset_with_offset_back_one_hour() {
        let date = Local.with_ymd_and_hms(2020, 1, 1, 18, 0, 0).unwrap();
        let date_utc = date.to_utc();
        // "2020-01-01T16:00:00Z"
        let offset = date.offset().fix().local_minus_utc() / 60;
        let delta = -(offset - 60); // -60

        let res_utc = get_datetime_for_offset(date, delta);
        // "2020-01-01T15:00:00Z"
        assert_eq!(date_utc.year(), res_utc.year());
        assert_eq!(date_utc.month(), res_utc.month());
        assert_eq!(date_utc.day(), res_utc.day());
        assert_eq!(date_utc.hour() - 1, res_utc.hour());
        assert_eq!(date_utc.minute(), res_utc.minute());
        assert_eq!(date_utc.second(), res_utc.second());
    }
    #[test]
    fn test_get_datetime_for_offset_with_offset_forward_one_hour() {
        let date = Local.with_ymd_and_hms(2020, 1, 1, 18, 0, 0).unwrap();
        let date_utc = date.to_utc();
        // "2020-01-01T16:00:00Z"
        let offset = date.offset().fix().local_minus_utc() / 60;
        let delta = -(offset + 60); // -180

        let res_utc = get_datetime_for_offset(date, delta);
        // "2020-01-01T17:00:00Z"
        assert_eq!(date_utc.year(), res_utc.year());
        assert_eq!(date_utc.month(), res_utc.month());
        assert_eq!(date_utc.day(), res_utc.day());
        assert_eq!(date_utc.hour() + 1, res_utc.hour());
        assert_eq!(date_utc.minute(), res_utc.minute());
        assert_eq!(date_utc.second(), res_utc.second());
    }
    #[test]
    fn test_get_datetime_for_offset_with_offset_back_two_hour() {
        let date = Local.with_ymd_and_hms(2020, 1, 1, 18, 0, 0).unwrap();
        let date_utc = date.to_utc();
        // "2020-01-01T16:00:00Z"
        let offset = date.offset().fix().local_minus_utc() / 60;
        let delta = -(offset - 120); // 0

        let res_utc = get_datetime_for_offset(date, delta);
        // "2020-01-01T14:00:00Z"
        assert_eq!(date_utc.year(), res_utc.year());
        assert_eq!(date_utc.month(), res_utc.month());
        assert_eq!(date_utc.day(), res_utc.day());
        assert_eq!(date_utc.hour() - 2, res_utc.hour());
        assert_eq!(date_utc.minute(), res_utc.minute());
        assert_eq!(date_utc.second(), res_utc.second());
    }
    #[test]
    fn test_get_datetime_for_offset_with_offset_forward_two_hour() {
        let date = Local.with_ymd_and_hms(2020, 1, 1, 18, 0, 0).unwrap();
        let date_utc = date.to_utc();
        // "2020-01-01T16:00:00Z"
        let offset = date.offset().fix().local_minus_utc() / 60;
        let delta = -(offset + 120); // -240

        let res_utc = get_datetime_for_offset(date, delta);
        // "2020-01-01T18:00:00Z"
        assert_eq!(date_utc.year(), res_utc.year());
        assert_eq!(date_utc.month(), res_utc.month());
        assert_eq!(date_utc.day(), res_utc.day());
        assert_eq!(date_utc.hour() + 2, res_utc.hour());
        assert_eq!(date_utc.minute(), res_utc.minute());
        assert_eq!(date_utc.second(), res_utc.second());
    }
}
