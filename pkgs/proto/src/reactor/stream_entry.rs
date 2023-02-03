use super::ParseError;
use redis::ErrorKind::TypeError;
use redis::{streams::StreamId, RedisError, Value as RedisValue};
use serde::{de::Deserialize, ser::Serialize};
use serde_json::{from_value, to_string as as_string, to_value};
use serde_json::{map::Map as JsonMap, Result as SerdeResult, Value as JsonValue};
use std::collections::BTreeMap;
use tap::Pipe;

pub trait StreamEntry: Send + Sync + Serialize + for<'de> Deserialize<'de> {
    /// Deserializes from a stringified key value map but only if every field is FromStr.
    fn from_stream_id(stream_id: &StreamId) -> Result<Self, ParseError> {
        (stream_id.map.iter())
            .map(|(key, value)| match value {
                RedisValue::Data(ref bytes) => (&bytes[..])
                    .pipe(std::str::from_utf8)
                    .map_err(ParseError::from)
                    .map(String::from)
                    .map(|value| (key.to_owned(), JsonValue::String(value))),
                _ => Err(RedisError::from((TypeError, "invalid response type for JSON")).into()),
            })
            .collect::<Result<Vec<(String, JsonValue)>, _>>()?
            .into_iter()
            .pipe(|iter| from_value::<Self>(JsonValue::Object(JsonMap::from_iter(iter))))?
            .pipe(Ok)
    }

    /// Serialize into a stringified field value mapping for XADD
    fn xadd_map(&self) -> Result<BTreeMap<String, String>, ParseError> {
        (to_value(self)?.as_object().into_iter())
            .flat_map(|map| {
                map.into_iter().filter_map(|(key, value)| match value {
                    JsonValue::Null => None,
                    JsonValue::Bool(value) => Some(Ok((key.to_owned(), value.to_string()))),
                    JsonValue::Number(value) => Some(Ok((key.to_owned(), value.to_string()))),
                    JsonValue::String(value) => Some(Ok((key.to_owned(), value.to_owned()))),
                    JsonValue::Array(v) => Some(as_string(v).map(|v| (key.to_owned(), v))),
                    JsonValue::Object(v) => Some(as_string(v).map(|v| (key.to_owned(), v))),
                })
            })
            .collect::<SerdeResult<Vec<(String, String)>>>()?
            .pipe(|v| BTreeMap::from_iter(v.into_iter()))
            .pipe(Ok)
    }
}

impl<T> StreamEntry for T where T: Send + Sync + Serialize + for<'de> Deserialize<'de> {}
