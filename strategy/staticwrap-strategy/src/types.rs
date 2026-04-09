use anyhow::Result;

/// 策略类型标识 - 轻量级枚举，不含实例数据
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StrategyType {
    ExecTester,
    // 新增策略在此添加...
}

impl StrategyType {
    /// 从字符串解析
    pub fn from_str(s: &str) -> Result<Self> {
        match s {
            "exec_tester" => Ok(StrategyType::ExecTester),
            _ => anyhow::bail!("Unknown strategy type: {}", s),
        }
    }

    /// 获取类型名称
    pub fn name(&self) -> &'static str {
        match self {
            StrategyType::ExecTester => "exec_tester",
        }
    }

    /// 列出所有可用类型
    pub fn all() -> Vec<Self> {
        vec![StrategyType::ExecTester]
    }

    /// 获取默认配置文件名
    pub fn default_config_file(&self) -> String {
        format!("{}.toml", self.name())
    }
}

impl std::fmt::Display for StrategyType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}
