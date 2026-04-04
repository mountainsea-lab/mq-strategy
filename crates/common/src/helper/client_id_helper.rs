use nautilus_model::identifiers::ClientId;
use serde::de::Error;
use serde::{Deserialize, Deserializer, Serializer};

// 自定义反序列化逻辑：处理 Option<ClientId>
pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<ClientId>, D::Error>
where
    D: Deserializer<'de>,
{
    let option: Option<String> = Option::deserialize(deserializer)?;
    match option {
        Some(id_str) => ClientId::new_checked(&id_str)
            .map(Some)
            .map_err(D::Error::custom),
        None => Ok(None),
    }
}

// 自定义序列化逻辑：处理 Option<ClientId>
pub fn serialize<S>(value: &Option<ClientId>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match value {
        Some(client_id) => serializer.serialize_str(&client_id.to_string()),
        None => serializer.serialize_none(),
    }
}
