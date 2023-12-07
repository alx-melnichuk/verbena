use chrono::{DateTime, TimeZone, Utc};
use serde::{self, Deserialize, Deserializer, Serializer};

const FORMAT: &'static str = "%Y-%m-%dT%H:%M:%S.%3fZ%z";

pub fn serialize<S>(date: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let buff = format!("{}", date.format(FORMAT));
    serializer.serialize_str(&buff)
}

pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
where
    D: Deserializer<'de>,
{
    let buff = String::deserialize(deserializer)?;
    Ok(if buff.len() == 0 {
        Utc::now()
    } else {
        let date = Utc.datetime_from_str(&buff, FORMAT).map_err(serde::de::Error::custom)?;
        date
    })
}
