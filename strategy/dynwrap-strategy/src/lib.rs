use anyhow::Context;
use nautilus_trading::{Strategy, StrategyConfig};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::to_writer_pretty;
use std::fmt::Debug;
use std::fs::{File, create_dir_all};
use std::path::Path;

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
    /// 从 JSON 文件加载
    fn from_file<P: AsRef<Path>>(path: P) -> anyhow::Result<Self>
    where
        Self: DeserializeOwned,
    {
        let file = File::open(&path)
            .with_context(|| format!("Failed to open file: {}", path.as_ref().display()))?;

        let config =
            serde_json::from_reader(file).context("Failed to deserialize JSON configuration")?;

        Ok(config)
    }

    /// 从 JSON 字符串加载（强烈推荐用于测试）
    fn from_str(json: &str) -> anyhow::Result<Self>
    where
        Self: DeserializeOwned,
    {
        let config = serde_json::from_str(json).context("Failed to deserialize JSON string")?;
        Ok(config)
    }

    /// 写入 JSON 文件（路径外部传入）
    fn write_to_file<P: AsRef<Path>>(&self, path: P) -> anyhow::Result<()>
    where
        Self: Serialize,
    {
        let path = path.as_ref();

        if let Some(parent) = path.parent() {
            create_dir_all(parent)
                .with_context(|| format!("Failed to create dir: {}", parent.display()))?;
        }

        let file = File::create(path)
            .with_context(|| format!("Failed to create file: {}", path.display()))?;

        to_writer_pretty(file, self).context("Failed to serialize config")?;

        Ok(())
    }

    /// 转 JSON 字符串（调试/日志）
    fn to_pretty_json(&self) -> anyhow::Result<String>
    where
        Self: Serialize,
    {
        Ok(serde_json::to_string_pretty(self)?)
    }
}
