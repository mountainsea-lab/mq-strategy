use nautilus_model::identifiers::InstrumentId;
use serde::de::Error;
use serde::{Deserialize, Deserializer, Serializer};
use std::str::FromStr;

// 自定义反序列化：支持 Option<Vec<InstrumentId>>
pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Vec<InstrumentId>>, D::Error>
where
    D: Deserializer<'de>,
{
    let option: Option<Vec<String>> = Option::deserialize(deserializer)?;
    match option {
        Some(ids) => {
            let instrument_ids = ids
                .into_iter()
                .map(|id_str| InstrumentId::from_str(&id_str))
                .collect::<Result<Vec<_>, _>>()
                .map_err(D::Error::custom)?;
            Ok(Some(instrument_ids))
        }
        None => Ok(None),
    }
}

// 自定义序列化：支持 Option<Vec<InstrumentId>>
pub fn serialize<S>(value: &Option<Vec<InstrumentId>>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match value {
        Some(instrument_ids) => {
            let ids: Vec<String> = instrument_ids.iter().map(|id| id.to_string()).collect();
            serializer.serialize_some(&ids)
        }
        None => serializer.serialize_none(),
    }
}
