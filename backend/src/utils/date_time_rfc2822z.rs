use chrono::{DateTime, Utc};
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
        let datetime_fixed =
            DateTime::parse_from_str(buff.as_str(), FORMAT).map_err(serde::de::Error::custom)?;
        let datetime_utc =
            DateTime::<Utc>::try_from(datetime_fixed).map_err(serde::de::Error::custom)?;
        datetime_utc
    })
}
