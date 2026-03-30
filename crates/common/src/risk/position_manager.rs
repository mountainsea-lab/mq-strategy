/// `PositionManager` 结构体用于根据凯利公式和波动率调整来计算最优仓位。
/// 它包含了基础仓位、最大/最小仓位、目标波动率和凯利系数等参数。
///
/// # 参数说明
/// - `base_position`: 基础仓位，通常基于凯利公式计算得出的初步仓位。
/// - `max_position`: 最大仓位，用于限制仓位的最大值，避免过度杠杆。
/// - `min_position`: 最小仓位，用于避免仓位过小，导致资金利用效率低。
/// - `volatility_target`: 目标波动率，控制仓位调整的目标波动范围。
/// - `kelly_multiplier`: 凯利系数，用于控制仓位的风险敞口，通常取 0.25 或 0.5 作为防止过度杠杆的措施。
pub struct PositionManager {
    pub base_position: f64,     // 基础仓位
    pub max_position: f64,      // 最大仓位
    pub min_position: f64,      // 最小仓位
    pub volatility_target: f64, // 目标波动率
    pub kelly_multiplier: f64,  // 凯利系数（通常0.25或0.5）
}

impl PositionManager {
    /// 创建一个新的 `PositionManager` 实例，初始化默认值
    ///
    /// # 返回
    /// - 返回一个 `PositionManager` 实例，初始化所有的参数。
    pub fn new() -> Self {
        Self {
            base_position: 0.1,      // 将基础仓位调整为 10%
            max_position: 0.2,       // 将最大仓位调整为 20%
            min_position: 0.05,      // 保持最小仓位为 5%
            volatility_target: 0.15, // 目标波动率保持为 15%
            kelly_multiplier: 0.25,  // 半凯利系数 0.25
        }
    }

    /// 根据凯利公式和波动率调整计算最优仓位
    ///
    /// # 参数
    /// - `win_rate`: 胜率 (p)，表示交易获胜的概率，范围 [0, 1]。
    /// - `win_loss_ratio`: 盈亏比 (b)，表示每次交易的期望收益与期望损失的比率。
    /// - `current_volatility`: 当前市场波动率，通常是历史波动率的一个估算值。
    ///
    /// # 返回
    /// - 最优仓位，根据凯利公式和波动率调整，限制在 `min_position` 和 `max_position` 之间。
    pub fn calculate_position(
        &self,
        win_rate: f64,
        win_loss_ratio: f64,
        current_volatility: f64,
    ) -> f64 {
        // 1. 凯利公式计算基础仓位，通常会根据实际情况乘以凯利系数（如0.25或0.5）以避免过度杠杆
        let kelly = kelly_fraction(win_rate, win_loss_ratio); // 计算凯利公式的基础仓位
        let kelly_position = kelly * self.kelly_multiplier; // 根据凯利系数调整仓位

        // 2. 根据当前波动率和目标波动率调整仓位
        let vol_adjusted = volatility_adjusted_position(
            kelly_position,         // 基础仓位
            current_volatility,     // 当前市场波动率
            self.volatility_target, // 目标波动率
        );

        // 3. 限制仓位在最大仓位和最小仓位之间，避免仓位过大或过小
        vol_adjusted.clamp(self.min_position, self.max_position)
    }
}

/// 根据凯利公式计算最优仓位
///
/// # 参数
/// - `win_rate`: 胜率 (p)，介于 0 到 1 之间的浮动数值，表示交易成功的概率
/// - `win_loss_ratio`: 盈亏比 (b)，表示每次交易的期望收益与期望损失的比率
///
/// # 返回
/// - 最优仓位 f*，范围通常为 [0, 1]，表示建议的投资比例
///
/// # 示例
/// 凯利公式计算: 胜率 55%，盈亏比 1.5
/// - f* = (0.55 * 1.5 - 0.45) / 1.5 = 0.25，意味着建议投资 25% 的资金
fn kelly_fraction(win_rate: f64, win_loss_ratio: f64) -> f64 {
    // 胜率 p
    let p = win_rate;
    // 败率 q = 1 - p
    let q = 1.0 - p;
    // 盈亏比 b
    let b = win_loss_ratio;

    // 计算凯利公式
    let kelly = (p * b - q) / b;

    // 限制仓位在合理范围 [0, 1] 之间
    kelly.clamp(0.0, 1.0)
}

/// 根据市场波动率动态调整仓位
///
/// # 参数
/// - `base_position`: 基础仓位，通常基于凯利公式计算的仓位
/// - `current_volatility`: 当前市场的波动率，单位通常为百分比
/// - `target_volatility`: 目标波动率，目标的市场波动率
///
/// # 返回
/// - 调整后的仓位
///
/// # 示例
/// 基础仓位 20%，当前波动率 25%，目标波动率 15%
/// - 调整后的仓位 = 20% × (15% / 25%) = 12%
fn volatility_adjusted_position(
    base_position: f64,
    current_volatility: f64,
    target_volatility: f64,
) -> f64 {
    // 计算波动率比率  波动大时减仓，波动小时加仓
    let volatility_ratio = target_volatility / current_volatility;

    // 计算调整后的仓位 限制调整范围
    let adjusted = base_position * volatility_ratio;

    // 限制仓位的调整范围，在 [50%, 200%] 之间
    adjusted.clamp(base_position * 0.5, base_position * 2.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 模拟一个策略的调用，其中 ATR 由外部传入
    #[test]
    fn test_position_sizer_with_external_atr() {
        // 假设策略已经计算并传入了 ATR
        let atr = 10.0; // 假设当前 ATR 波动率为 12%

        // 设置 PositionManager，并传入当前的 ATR 作为目标波动率
        let position_sizer = PositionManager {
            volatility_target: atr,   // 使用外部传入的 ATR 值
            ..PositionManager::new()  // 使用默认的其他值
        };

        // 胜率和盈亏比
        let win_rate = 0.6;
        let win_loss_ratio = 1.5;
        let current_volatility = atr; // 当前波动率即为传入的 ATR

        // 计算仓位
        let calculated_position =
            position_sizer.calculate_position(win_rate, win_loss_ratio, current_volatility);

        // 输出计算的仓位大小
        println!("计算出来的仓位大小: {:.2}%", calculated_position * 100.0);
        // 断言仓位在合理范围内
        assert!(calculated_position >= position_sizer.min_position);
        assert!(calculated_position <= position_sizer.max_position);
    }
}
