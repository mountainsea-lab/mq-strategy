use crate::config::ExecTesterConfig;
use crate::strategy::ExecTester;
use anyhow::Result;
use dynwrap_strategy::{SConfigSerializable, StrategyExt};

pub mod config;
pub mod strategy;

/// 暴露一个工厂函数来实例化策略
#[unsafe(no_mangle)]
pub extern "C" fn create_strategy(config_path: &str) -> Result<Box<dyn StrategyExt>> {
    let mut config = ExecTesterConfig::from_json(config_path)?;
    config.base.auto_assign_fields();
    Ok(Box::new(ExecTester::new(config)))
}

//
// #[unsafe(no_mangle)]
// pub fn create_strategy(config_path: &str) -> Result<Box<dyn Strategy>> {
//     let config = ExecTesterConfig::from_json(config_path)?;
//     let strategy = ExecTester::new(config.clone());
//     let wrapper = DynStrategyWrapper::new(Box::new(strategy), Box::new(config));
//     Ok(Box::new(wrapper)) // 返回包装后的策略
// }
