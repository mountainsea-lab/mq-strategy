use nautilus_trading::StrategyConfig;
use std::fmt::Debug;

pub mod strategy_wrapper;

pub trait SConfig: Debug {
    fn base(&self) -> &StrategyConfig;
}
