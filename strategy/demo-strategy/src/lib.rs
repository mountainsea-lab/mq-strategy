use crate::config::ExecTesterConfig;
use crate::strategy::ExecTester;
use anyhow::Result;
use nautilus_trading::Strategy;

pub mod config;
pub mod strategy;

/// 暴露一个工厂函数来实例化策略
#[unsafe(no_mangle)]
pub fn create_strategy(config_path: &str) -> Result<Box<dyn Strategy>> {
    let config = ExecTesterConfig::from_json(config_path)?;
    Ok(Box::new(ExecTester::new(config)))
}
