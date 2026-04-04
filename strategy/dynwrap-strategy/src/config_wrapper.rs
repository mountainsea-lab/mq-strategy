use mq_strategy_common::helper::{instrument_id_opvec_helper, strategy_id_helper};
use nautilus_model::identifiers::{InstrumentId, StrategyId};
use nautilus_trading::StrategyConfig;
use serde::de::Visitor;
use serde::{Deserialize, Deserializer, Serialize};
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomStrategyConfig {
    #[serde(flatten)] // 展开 StrategyConfig 中的字段
    pub strategy_config: StrategyConfig,

    #[serde(with = "strategy_id_helper")]
    pub strategy_id: Option<StrategyId>,

    #[serde(with = "instrument_id_opvec_helper")]
    pub external_order_claims: Option<Vec<InstrumentId>>,
}

impl Default for CustomStrategyConfig {
    fn default() -> Self {
        Self {
            strategy_config: StrategyConfig::default(),
            strategy_id: None,
            external_order_claims: None,
        }
    }
}

impl CustomStrategyConfig {
    // Custom method to automatically assign the values to StrategyConfig
    pub fn auto_assign_fields(&mut self) {
        if let Some(strategy_id_value) = self.strategy_id.clone() {
            self.strategy_config.strategy_id = Some(strategy_id_value);
        }

        if let Some(external_order_claims_value) = self.external_order_claims.clone() {
            self.strategy_config.external_order_claims = Some(external_order_claims_value);
        }
    }
}
