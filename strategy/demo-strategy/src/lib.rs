use crate::config::ExecTesterConfig;
use crate::strategy::ExecTester;
use dynwrap_strategy::export_strategy;

pub mod config;
pub mod strategy;

// ========== 一行宏生成所有导出 ==========

export_strategy! {
    strategy_type: ExecTester,
    config_type: ExecTesterConfig,
    config_process: |mut config: ExecTesterConfig| {
        config.base.auto_assign_fields();
        config
    },
}
