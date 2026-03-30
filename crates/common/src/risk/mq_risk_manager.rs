use crate::risk::position_manager::PositionManager;
use log::{debug, info};
use nautilus_model::data::{Bar, QuoteTick};
use nautilus_model::enums::PositionSide;
use nautilus_model::identifiers::InstrumentId;
use nautilus_model::instruments::Instrument;
use nautilus_model::instruments::InstrumentAny;
use nautilus_model::position::Position;
use nautilus_model::types::{Price, Quantity};
use time::Date;

/// 风控：震荡 + 支撑压力位风险控制
/// 该结构体用于定义和计算基于风险的仓位管理和日损失限制，确保每笔交易和日内总损失都在预设范围内。
///   1. 完善开仓逻辑 仓位数控制,下单金额
///   2. 完善平仓逻辑判断
///      平仓逻辑: 盈亏比判断(开空和开多满足盈利平仓), 止损: 跌破支撑位/突破压力位,结合震荡指标值判断
///   3. 校验当日亏损超过限制不允许交易
#[derive(Debug)]
pub struct MqRiskManager {
    /// 每笔交易最大风险比例
    /// 这表示每笔交易所能承受的最大损失，占总账户资金的比例。
    /// 例如，0.01 表示每笔交易的最大损失为账户资金的 1%。
    pub max_risk_per_trade: f64,

    /// 最大日损失比例
    /// 这是指单日内允许的最大损失，占总账户资金的比例。
    /// 例如，0.05 表示单日最大损失为账户资金的 5%。
    pub max_daily_loss: f64,

    /// 最大总风险敞口
    /// 表示账户可以承受的最大总风险敞口，占总资金的比例。
    /// 例如，0.10 表示最大总风险敞口为账户资金的 10%。
    pub max_total_risk: f64,

    /// 最大仓位控制
    /// 策略运行同时开单的数量
    /// 例如，10 表示策略同时只能开10单
    pub max_position_size: usize,

    /// ATR止损倍数
    /// 这个参数用于计算基于平均真实波动范围（ATR）的止损幅度。
    /// ATR越高，止损幅度越大，反之亦然。
    /// 一般的做法是使用2倍ATR作为止损幅度。
    pub atr_multiplier: f64,

    /// 最小交易量
    /// 确保每笔交易的交易量不低于交易所要求的最小交易量。
    /// 这通常是交易所的最低交易数量，确保符合交易所的交易要求。
    pub min_trade_size: f64,

    /// 今日累计损益
    /// 用于记录当天的累计损益（Profit and Loss），以便计算日损失限制。
    /// 当日损益达到最大限制时，策略会停止交易。
    daily_pnl: f64,

    /// 上次重置的日期
    /// 用于跟踪上一交易日的日期，确保每日损益的重置。
    /// 如果是新的一天，则重置 `daily_pnl`。
    last_reset_date: Option<Date>,

    /// 盈亏比例
    /// 用于平仓判断
    /// 默认盈亏比 1 1.5 2 ...
    win_loss_ratio: f64,
    /// 杠杆倍数
    /// 用于杠杆
    /// 默认 5x
    leverage: f64,
}

impl MqRiskManager {
    /// 创建一个新的 `RiskChandelier` 实例
    ///
    /// # 参数
    /// - `max_risk_per_trade`: 每笔交易的最大风险比例
    /// - `max_daily_loss`: 每日最大损失比例
    /// - `max_total_risk`: 最大总风险敞口
    /// - `atr_multiplier`: ATR止损倍数
    /// - `min_trade_size`: 最小交易量
    ///
    /// # 返回
    /// 返回一个 `RiskChandelier` 实例。
    pub fn new(
        max_risk_per_trade: f64,
        max_daily_loss: f64,
        max_total_risk: f64,
        atr_multiplier: f64,
        min_trade_size: f64,
    ) -> Self {
        Self {
            max_risk_per_trade,
            max_daily_loss,
            max_total_risk,
            max_position_size: 0,
            atr_multiplier,
            min_trade_size,
            daily_pnl: 0.0,
            last_reset_date: None,
            win_loss_ratio: 1.0,
            leverage: 5.0,
        }
    }

    /// 创建一个带有默认参数的 `RiskChandelier` 实例
    ///
    /// # 返回
    /// 返回一个 `RiskChandelier` 实例，其中包含推荐的默认风控参数。
    pub fn default() -> Self {
        Self {
            max_risk_per_trade: 0.01, // 每笔交易最大风险为账户资金的 1%

            max_daily_loss: 0.05, // 单日最大损失为账户资金的 5%

            max_total_risk: 0.10, // 最大总风险敞口为账户资金的 10%

            max_position_size: 10, // 最大开单数量

            atr_multiplier: 2.0, // 基于 2 倍 ATR 设置止损幅度

            min_trade_size: 0.01, // 默认每次交易 base

            daily_pnl: 0.0,

            last_reset_date: None,

            win_loss_ratio: 1.5,

            leverage: 5.0,
        }
    }

    /// 检查是否可以开新单
    ///
    /// # 参数
    /// - `all_positions`: 当前所有仓位
    /// - `position_side`: 当前要开盘的方向（`Long` 或 `Short`）
    ///
    /// # 返回
    /// 如果符合仓位限制，返回 `true`，否则返回 `false`。
    pub fn can_open_position(
        &self,
        all_positions: Vec<Position>,
        instrument_id: &InstrumentId,
        position_side: PositionSide,
    ) -> bool {
        // 检查总仓位限制
        if all_positions.len() >= self.max_position_size {
            info!("{}当前总仓位数已达最大限制，无法开仓", instrument_id);
            return false;
        }

        // 根据方向过滤现有仓位
        let side_positions: Vec<&Position> = all_positions
            .iter()
            .filter(|pos| pos.side == position_side)
            .collect();

        // 检查单方向仓位限制
        let max_per_side = self.max_position_size / 2;
        if side_positions.len() >= max_per_side {
            info!(
                "{},当前{:?}方向仓位数已达最大限制 {}/{}，无法开仓",
                instrument_id,
                position_side,
                side_positions.len(),
                max_per_side
            );
            return false;
        }

        // =========================
        // 同资产 + 同方向限制
        // =========================
        let exists_same_instrument = all_positions
            .iter()
            .any(|pos| pos.instrument_id == *instrument_id && pos.side == position_side);

        if exists_same_instrument {
            debug!("❌交易资产{:?} 已存在仓位，禁止重复开仓", instrument_id);
            return false;
        }

        true
    }

    /// 检查是否需要止损
    pub fn check_stop_loss(&self, last_bar: &Bar, position: &Position) -> bool {
        // 当前仓位开单价格
        let entry_price = position.avg_px_open;
        // 止损价格
        let stop_loss = self.calculate_stop_loss(entry_price, position.side, last_bar);
        match position.side {
            PositionSide::Long => {
                // 多单平仓逻辑：当前价格小于止损价或大于止盈价
                if last_bar.close.as_f64() <= stop_loss {
                    debug!("平仓信号触发：头仓位止损");
                    return true; // 满足平仓条件，返回 `true`
                }
            }
            PositionSide::Short => {
                // 空单平仓逻辑：当前价格大于止损价或小于止盈价
                if last_bar.close.as_f64() >= stop_loss {
                    debug!("平仓信号触发：空头仓位止损");
                    return true; // 满足平仓条件，返回 `true`
                }
            }
            _ => {}
        }

        false // 如果没有满足任何平仓条件，返回 `false`
    }

    /// 检查是否需要止盈
    pub fn check_stop_profit(
        &self,
        last_bar: &Bar,
        quote: &QuoteTick,
        position: &Position,
        atr: f64,
    ) -> bool {
        // 当前仓位开单价格
        let entry_price = position.avg_px_open;
        // 止损价格
        let stop_loss = self.calculate_stop_loss(entry_price, position.side, last_bar);
        let take_profit = self.calculate_take_profit(
            entry_price,
            stop_loss,
            self.win_loss_ratio,
            position.side,
            atr,
        );

        match position.side {
            PositionSide::Long => {
                // 多单平仓逻辑：当前价格小于止损价或大于止盈价
                if quote.ask_price.as_f64() >= take_profit {
                    info!("平仓信号触发：多头仓位止盈");
                    return true; // 满足平仓条件，返回 `true`
                }
            }
            PositionSide::Short => {
                // 空单平仓逻辑：当前价格大于止损价或小于止盈价
                if quote.bid_price.as_f64() <= take_profit {
                    info!("平仓信号触发：空头仓位止盈");
                    return true; // 满足平仓条件，返回 `true`
                }
            }
            _ => {}
        }

        false // 如果没有满足任何平仓条件，返回 `false`
    }

    /// 计算止损价格：根据开仓价格的高低价或者固定百分比设置止损
    pub fn calculate_stop_loss(&self, entry_price: f64, side: PositionSide, last_bar: &Bar) -> f64 {
        match side {
            PositionSide::Long => {
                // 多单止损：参考前一根K线最低价，或者 entry_price * (1 - 0.01) 1%止损
                let low_price = last_bar.low.as_f64();
                (entry_price * 0.99).min(low_price)
            }
            PositionSide::Short => {
                // 空单止损：参考前一根K线最高价，或者 entry_price * (1 + 0.01) 1%止损
                let high_price = last_bar.high.as_f64();
                (entry_price * 1.01).max(high_price)
            }
            _ => entry_price,
        }
    }

    /// 计算动态止盈：基于盈亏比和 ATR 来设置止盈位置
    fn calculate_take_profit(
        &self,
        entry_price: f64,
        stop_loss: f64,
        win_loss_ratio: f64,
        side: PositionSide,
        atr: f64,
    ) -> f64 {
        let risk_distance = (entry_price - stop_loss).abs(); // 计算止损和入场价的差值（即风险距离）
        let take_profit_distance = risk_distance * win_loss_ratio; // 基于盈亏比调整止盈距离

        // 设定最大 ATR 调整的影响范围，避免止盈过远
        const MAX_ATR_ADJUSTMENT: f64 = 1.0; // 最大调整因子（例如，最多增加1倍ATR）

        // 基于市场波动（ATR）调整止盈距离，且不超过最大调整值
        let adjusted_take_profit_distance = take_profit_distance + atr.min(MAX_ATR_ADJUSTMENT);

        match side {
            PositionSide::Long => entry_price + adjusted_take_profit_distance, // 多头：止盈 = 入场价 + 调整后的止盈距离
            PositionSide::Short => entry_price - adjusted_take_profit_distance, // 空头：止盈 = 入场价 - 调整后的止盈距离
            _ => entry_price, // 如果不是多头或空头，直接返回入场价
        }
    }

    /// 计算仓位大小（返回 base 数量）凯利公式计算仓位
    /// # 参数
    /// - `instrument`: 交易base资产
    /// - `entry_price`: 当前进场价格
    /// - `stop_loss`: 当前止损价格
    /// - `free_balance`: 可用资金（USDT）
    ///
    /// # 返回
    /// 返回可以下单的 base 数量，已考虑风险、杠杆、最小/最大交易量。
    pub fn calculate_position(
        &mut self,
        instrument: &InstrumentAny,
        free_balance: f64,
        price: &Price,
        atr: f64,
    ) -> Option<Quantity> {
        // 设置 PositionManager，并传入当前的 ATR 作为目标波动率
        let position_sizer = PositionManager {
            volatility_target: atr,   // 使用外部传入的 ATR 值
            ..PositionManager::new()  // 使用默认的其他值
        };

        // 胜率和盈亏比 todo 先默认 后续策略稳定进行统计
        let win_rate = 0.60;
        let current_volatility = atr; // 当前波动率即为传入的 ATR

        // 计算仓位
        let calculated_position =
            position_sizer.calculate_position(win_rate, self.win_loss_ratio, current_volatility);
        // 输出计算的仓位大小
        println!(
            "当前资金余额{},计算出来的仓位大小: {:.2}%",
            free_balance,
            calculated_position * 100.0
        );
        // 使用当前市场价格作为开盘价格
        let entry_price = price.as_f64(); // Use the latest market bar for entry price

        // 计算原始仓位大小（calculated_position * free_balance）
        let raw_quote_size = calculated_position * free_balance;

        // 设置最大仓位为 free_balance 的 1/10
        let max_quote_size = free_balance / 10.0;

        // 限制仓位不超过 max_quantity_size
        let quote_size = raw_quote_size.min(max_quote_size);
        let quantity_size = quote_size * self.leverage / entry_price;

        Some(instrument.make_qty(quantity_size, None))
    }
}
