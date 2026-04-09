use crate::types::StrategyType;
use anyhow::{Context, Result};
use dynwrap_strategy::SConfigSerializable;
use mq_demo_strategy::config::ExecTesterConfig;
use mq_demo_strategy::strategy::ExecTester;

/// 策略类型枚举 - 新增策略只需在此添加变体
pub enum StrategyWrapper {
    ExecTester(ExecTester),
}

impl StrategyWrapper {
    /// 根据类型和配置路径创建策略实例
    pub fn create(stype: StrategyType, config_path: &str) -> Result<Self> {
        match stype {
            StrategyType::ExecTester => {
                let config = ExecTesterConfig::from_file(config_path).with_context(|| {
                    format!("Failed to load ExecTester config from: {}", config_path)
                })?;
                Ok(StrategyWrapper::ExecTester(ExecTester::new(config)))
            }
        }
    }

    /// 获取策略类型
    pub fn strategy_type(&self) -> StrategyType {
        match self {
            StrategyWrapper::ExecTester(_) => StrategyType::ExecTester,
        }
    }

    /// 消费 wrapper，返回 ExecTester 所有权
    pub fn into_exec_tester(self) -> Option<ExecTester> {
        match self {
            StrategyWrapper::ExecTester(s) => Some(s),
            _ => None,
        }
    }
}
