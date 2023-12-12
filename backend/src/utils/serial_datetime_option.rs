use chrono::{DateTime, SecondsFormat, Utc};
use serde::{self, Deserialize, Deserializer, Serializer};

// const FORMAT: &'static str = "%Y-%m-%dT%H:%M:%S.%3fZ"; // "YYYY-mm-ddTHH:MM:SS.fffZ"

pub fn serialize<S>(date: &Option<DateTime<Utc>>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match date {
        Some(date) => {
            // rfc3339: '2023-11-14T14:01:56.323Z'
            let buff = date.to_rfc3339_opts(SecondsFormat::Millis, true);
            // let buff = format!("{}", date.format(FORMAT));
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
        // rfc3339: '2023-11-14T14:01:56.323Z'
        let datetime_fixed =
            DateTime::parse_from_rfc3339(buff.as_str()).map_err(serde::de::Error::custom)?;
        // let datetime_fixed =
        //     DateTime::parse_from_str(buff.as_str(), FORMAT).map_err(serde::de::Error::custom)?;
        let datetime_utc =
            DateTime::<Utc>::try_from(datetime_fixed).map_err(serde::de::Error::custom)?;
        Some(datetime_utc)
    })
}
