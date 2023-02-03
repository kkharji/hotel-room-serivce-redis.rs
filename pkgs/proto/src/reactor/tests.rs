use super::StreamEntry;
use decimal::d128;
use redis::{streams::StreamId, RedisError};
use serde::{Deserialize, Serialize};
use serde_aux::prelude::*;
use serde_with::serde_as;
use std::collections::{BTreeMap, HashMap};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Author {
    name: String,
    tags: Vec<String>,
}

impl TryFrom<&str> for Author {
    type Error = serde_json::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        serde_json::from_str(value)
    }
}

impl TryFrom<String> for Author {
    type Error = serde_json::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        serde_json::from_str(value.as_str())
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum CreateLorem {
    Sentences {
        #[serde(deserialize_with = "deserialize_number_from_string")]
        count: i64,
    },
    Paragraphs {
        #[serde(deserialize_with = "deserialize_number_from_string")]
        count: i64,
        #[serde(deserialize_with = "deserialize_number_from_string")]
        sentences_per_paragraph: i64,
    },
}

#[serde_as]
#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "op", rename_all = "camelCase")]
pub enum Op {
    CreateLorem(CreateLorem),
    CreatePost {
        #[serde(deserialize_with = "deserialize_number_from_string")]
        id: i64,
        content: String,
        #[serde_as(as = "serde_with::PickFirst<(_, serde_with::json::JsonString)>")]
        author: Author,
    },
    SetWinRate {
        rate: d128,
    },
}

#[test]
fn deserializes_from_stream_id() -> Result<(), RedisError> {
    let mut map: HashMap<String, redis::Value> = HashMap::new();
    map.insert(
        "op".into(),
        redis::Value::Data("createLorem".as_bytes().to_owned()),
    );
    map.insert(
        "type".into(),
        redis::Value::Data("sentences".as_bytes().to_owned()),
    );
    map.insert(
        "count".into(),
        redis::Value::Data("100".as_bytes().to_owned()),
    );

    let stream_id = StreamId {
        id: "0-0".into(),
        map,
    };

    let value = Op::from_stream_id(&stream_id)?;

    assert_eq!(
        value,
        Op::CreateLorem(CreateLorem::Sentences { count: 100 })
    );

    Ok(())
}

#[test]
fn deserializes_nested_structs() -> Result<(), RedisError> {
    let author = Author {
        name: String::from("Bajix"),
        tags: vec!["Rustacean".into()],
    };

    let mut map: HashMap<String, redis::Value> = HashMap::new();
    map.insert(
        "op".into(),
        redis::Value::Data("createPost".as_bytes().to_owned()),
    );
    map.insert("id".into(), redis::Value::Data("1".as_bytes().to_owned()));
    map.insert(
        "content".into(),
        redis::Value::Data("Hello World!".as_bytes().to_owned()),
    );
    map.insert(
        "author".into(),
        redis::Value::Data(
            serde_json::to_string(&author)
                .unwrap()
                .as_bytes()
                .to_owned(),
        ),
    );

    let stream_id = StreamId {
        id: "0-0".into(),
        map,
    };

    let value = Op::from_stream_id(&stream_id)?;

    assert_eq!(
        value,
        Op::CreatePost {
            id: 1,
            content: String::from("Hello World!"),
            author
        }
    );

    Ok(())
}

#[test]
fn deserializes_d128() -> Result<(), RedisError> {
    let mut map: HashMap<String, redis::Value> = HashMap::new();
    map.insert(
        "op".into(),
        redis::Value::Data("setWinRate".as_bytes().to_owned()),
    );
    map.insert(
        "rate".into(),
        redis::Value::Data(".5".as_bytes().to_owned()),
    );

    let stream_id = StreamId {
        id: "0-0".into(),
        map,
    };

    let value = Op::from_stream_id(&stream_id)?;

    assert_eq!(value, Op::SetWinRate { rate: d128!(0.5) });

    Ok(())
}

#[test]
fn into_redis_btreemap() -> Result<(), RedisError> {
    let op = Op::CreatePost {
        id: 1,
        content: String::from("Hello World!"),
        author: Author {
            name: String::from("Bajix"),
            tags: vec!["Rustacean".into()],
        },
    };

    let data: Vec<(String, String)> = vec![
        (
            "author".into(),
            "{\"name\":\"Bajix\",\"tags\":[\"Rustacean\"]}".into(),
        ),
        ("content".into(), "Hello World!".into()),
        ("id".into(), "1".into()),
        ("op".into(), "createPost".into()),
    ];

    let data: BTreeMap<String, String> = BTreeMap::from_iter(data.into_iter());

    assert_eq!(op.xadd_map()?, data);

    Ok(())
}
