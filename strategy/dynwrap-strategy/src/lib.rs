use anyhow::Context;
use nautilus_trading::{Strategy, StrategyConfig};
use serde::{Deserialize, Serialize};
use serde_json::to_writer_pretty;
use std::env;
use std::fmt::Debug;
use std::fs::{File, create_dir_all};

pub mod config_wrapper;
pub mod strategy_wrapper;
pub mod strategy_wrapper_ffi;

pub use strategy_wrapper_ffi::StrategyVTable;

pub use strategy_wrapper_ffi::DynStrategyWrapper;
mod macros;

pub trait StrategyExt: Strategy {
    fn s_config(&self) -> Box<dyn SConfig>;
}

pub trait SConfig: Debug {
    fn base(&self) -> &StrategyConfig;
}

pub trait SConfigSerializable {
    fn from_json(path: &str) -> anyhow::Result<Self>
    where
        Self: for<'de> Deserialize<'de> + Serialize,
    {
        let file = File::open(path).context(format!("Failed to open file: {}", path))?;
        let config =
            serde_json::from_reader(file).context("Failed to deserialize JSON configuration")?;
        Ok(config)
    }

    fn write_to_json(&self) -> anyhow::Result<()>
    where
        Self: Serialize,
    {
        // 获取当前工作目录
        let current_dir =
            env::current_dir().context("Failed to get the current working directory")?;

        // 获取自定义的路径或使用默认路径
        let config_dir = current_dir
            .join("strategy")
            .join("demo-strategy")
            .join("config");

        // 如果目录不存在则创建
        create_dir_all(&config_dir).context("Failed to create the config directory")?;

        // 目标文件路径
        let file_path = config_dir.join("default.json");

        // 打开文件并序列化配置
        let file = File::create(&file_path)
            .context(format!("Failed to create file: {}", file_path.display()))?;
        to_writer_pretty(file, &self).context("Failed to serialize the configuration into JSON")?;

        Ok(())
    }
}
