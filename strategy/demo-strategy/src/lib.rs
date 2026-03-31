use crate::config::ExecTesterConfig;
use crate::strategy::ExecTester;
use nautilus_trading::Strategy;

pub mod config;
pub mod strategy;

// /// 暴露一个工厂函数来实例化策略
// #[unsafe(no_mangle)]
// pub extern "C" fn create_strategy(config: ExecTesterConfig) -> Box<dyn Strategy> {
//     Box::new(ExecTester::new(config))
// }
