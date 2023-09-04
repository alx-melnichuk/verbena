use chrono::{DateTime, TimeZone, Utc};
use serde::{self, Deserialize, Deserializer, Serializer};

const FORMAT: &'static str = "%Y-%m-%dT%H:%M:%S.%3fZ%z";

pub fn serialize<S>(date: &Option<DateTime<Utc>>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match date {
        Some(date) => {
            let buff = format!("{}", date.format(FORMAT));
            serializer.serialize_str(&buff)
        }
        _ => unreachable!(),
    }
}

pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<DateTime<Utc>>, D::Error>
where
    D: Deserializer<'de>,
{
    let buff = String::deserialize(deserializer)?;
    Ok(if buff.is_empty() {
        None
    } else {
        let date = Utc
            .datetime_from_str(&buff, FORMAT)
            .map_err(serde::de::Error::custom)?;
        Some(date)
    })
}
