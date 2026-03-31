use anyhow::Result;
use nautilus_model::types::Quantity;
use serde::{Deserialize, Deserializer, Serializer};

pub fn deserialize<'de, D>(deserializer: D) -> Result<Quantity, D::Error>
where
    D: Deserializer<'de>,
{
    let qty_str: String = Deserialize::deserialize(deserializer)?; // Deserialize as a string
    qty_str
        .parse::<String>()
        .map(Quantity::from) // Convert the parsed Decimal into a Quantity type
        .map_err(|e| serde::de::Error::custom(format!("Failed to parse Quantity: {}", e)))
}

pub fn serialize<S>(quantity: &Quantity, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let qty_str = quantity.to_string(); // Assuming `Quantity` can be represented as a string
    serializer.serialize_str(&qty_str)
}
