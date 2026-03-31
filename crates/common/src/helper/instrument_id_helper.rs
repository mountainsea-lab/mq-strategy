use nautilus_model::identifiers::InstrumentId;
use serde::de::Visitor;
use serde::{Deserializer, Serializer, de};
use std::fmt;
use std::str::FromStr;

pub fn deserialize<'de, D>(deserializer: D) -> Result<InstrumentId, D::Error>
where
    D: Deserializer<'de>,
{
    struct InstrumentIdVisitor;

    impl<'de> Visitor<'de> for InstrumentIdVisitor {
        type Value = InstrumentId;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            write!(formatter, "a valid InstrumentId string")
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            InstrumentId::from_str(value)
                .map_err(|e| E::custom(format!("Invalid InstrumentId: {}", e)))
        }
    }

    deserializer.deserialize_str(InstrumentIdVisitor)
}
pub fn serialize<S>(instrument_id: &InstrumentId, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&instrument_id.to_string())
}
