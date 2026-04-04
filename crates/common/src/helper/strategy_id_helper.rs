use nautilus_model::identifiers::StrategyId;
use serde::de::Error;
use serde::{Deserialize, Deserializer, Serializer};

// 自定义反序列化逻辑：处理 Option<StrategyId>
pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<StrategyId>, D::Error>
where
    D: Deserializer<'de>,
{
    let option: Option<String> = Option::deserialize(deserializer)?;
    match option {
        Some(id_str) => StrategyId::new_checked(&id_str)
            .map(Some)
            .map_err(D::Error::custom),
        None => Ok(None),
    }
}

// 自定义序列化逻辑：处理 Option<StrategyId>
pub fn serialize<S>(value: &Option<StrategyId>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match value {
        Some(strategy_id) => serializer.serialize_str(&strategy_id.to_string()),
        None => serializer.serialize_none(),
    }
}
