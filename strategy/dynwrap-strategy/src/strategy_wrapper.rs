use crate::{SConfig, StrategyExt};
use anyhow::Result;
use nautilus_common::actor::{DataActor, DataActorCore};
use nautilus_common::timer::TimeEvent;
use nautilus_model::data::{
    Bar, IndexPriceUpdate, MarkPriceUpdate, OrderBookDeltas, QuoteTick, TradeTick,
};
use nautilus_model::identifiers::InstrumentId;
use nautilus_model::instruments::InstrumentAny;
use nautilus_model::orderbook::OrderBook;
use nautilus_trading::{Strategy, StrategyCore};
use std::fmt;
use std::ops::{Deref, DerefMut};
use log::info;

// 包装器结构体，持有一个 `Box<dyn Strategy>`
#[repr(C)]
pub struct DynStrategyWrapper {
    strategy: Box<dyn StrategyExt>,
}

impl DynStrategyWrapper {
    // 创建包装器的构造函数
    pub fn new(strategy: Box<dyn StrategyExt>) -> Result<Self> {
        if strategy.config.actor_id.is_none() {
            return Err(anyhow::anyhow!("Strategy is None").into());
        }
        info!("Creating DynStrategyWrapper");
        Ok(DynStrategyWrapper { strategy })
    }
}

impl fmt::Debug for DynStrategyWrapper {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("DynStrategyWrapper")
            .field("strategy", &"dynstrategy")
            // .field("config", &self.config)
            .finish()
    }
}

// 通过 Deref 和 DerefMut 将方法调用转发到实际的策略
impl Deref for DynStrategyWrapper {
    type Target = DataActorCore;

    fn deref(&self) -> &Self::Target {
        // 转发引用
        self.strategy.core()
    }
}

impl DerefMut for DynStrategyWrapper {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // 转发可变引用
        self.strategy.core_mut()
    }
}

// Implement StrategyExt for the wrapper, forwarding to the wrapped strategy's `s_config`
impl StrategyExt for DynStrategyWrapper {
    fn s_config(&self) -> Box<dyn SConfig> {
        self.strategy.s_config() // Forward the call to the wrapped strategy
    }
}

// 实现 Strategy trait，所有方法转发到实际的策略
impl Strategy for DynStrategyWrapper {
    fn core(&self) -> &StrategyCore {
        info!("Accessing core of strategy");
        self.strategy.core()
    }

    fn core_mut(&mut self) -> &mut StrategyCore {
        self.strategy.core_mut()
    }

    fn external_order_claims(&self) -> Option<Vec<InstrumentId>> {
        self.s_config().base().external_order_claims.clone()
    }
}

// 实现 DataActor trait，直接转发调用到底层 strategy
impl DataActor for DynStrategyWrapper {
    fn on_start(&mut self) -> Result<()> {
        // 显式调用 DataActor 的 on_start 方法
        DataActor::on_start(&mut *self.strategy)
    }

    fn on_stop(&mut self) -> Result<()> {
        // 显式调用 DataActor 的 on_stop 方法
        DataActor::on_stop(&mut *self.strategy)
    }

    fn on_time_event(&mut self, event: &TimeEvent) -> Result<()> {
        // 显式调用 DataActor 的 on_time_event 方法
        DataActor::on_time_event(&mut *self.strategy, event)
    }

    fn on_instrument(&mut self, instrument: &InstrumentAny) -> Result<()> {
        // 显式调用 DataActor 的 on_instrument 方法
        DataActor::on_instrument(&mut *self.strategy, instrument)
    }

    fn on_book_deltas(&mut self, deltas: &OrderBookDeltas) -> Result<()> {
        // 显式调用 DataActor 的 on_book_deltas 方法
        DataActor::on_book_deltas(&mut *self.strategy, deltas)
    }

    fn on_book(&mut self, book: &OrderBook) -> Result<()> {
        // 显式调用 DataActor 的 on_book 方法
        DataActor::on_book(&mut *self.strategy, book)
    }

    fn on_quote(&mut self, quote: &QuoteTick) -> Result<()> {
        // 直接调用 strategy 的 on_quote 方法
        self.strategy.on_quote(quote)
    }

    fn on_trade(&mut self, trade: &TradeTick) -> Result<()> {
        // 直接调用 strategy 的 on_trade 方法
        self.strategy.on_trade(trade)
    }

    fn on_bar(&mut self, bar: &Bar) -> Result<()> {
        // 直接调用 strategy 的 on_bar 方法
        self.strategy.on_bar(bar)
    }

    fn on_mark_price(&mut self, mark_price: &MarkPriceUpdate) -> Result<()> {
        // 直接调用 strategy 的 on_mark_price 方法
        self.strategy.on_mark_price(mark_price)
    }

    fn on_index_price(&mut self, index_price: &IndexPriceUpdate) -> Result<()> {
        // 直接调用 strategy 的 on_index_price 方法
        self.strategy.on_index_price(index_price)
    }
}
