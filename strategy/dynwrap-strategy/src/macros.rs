// dynwrap-strategy/src/macros.rs

/// 为具体策略生成 vtable 函数和导出
#[macro_export]
macro_rules! export_strategy {
    (
        strategy_type: $strategy:ty,
        config_type: $config:ty,
        $(config_process: $process_fn:expr,)?
    ) => {
        use std::ffi::{c_void, CStr};

        // ========== VTable 函数实现（不需要 no_mangle）==========

        pub extern "C" fn strategy_destroy(ptr: *mut c_void) {
            unsafe {
                let _ = Box::from_raw(ptr as *mut $strategy);
            }
        }

        pub extern "C" fn strategy_core(ptr: *const c_void) -> *const nautilus_trading::StrategyCore {
            unsafe {
                let s = &*(ptr as *const $strategy);
                <$strategy as nautilus_trading::Strategy>::core(s)
            }
        }

        pub extern "C" fn strategy_core_mut(ptr: *mut c_void) -> *mut nautilus_trading::StrategyCore {
            unsafe {
                let s = &mut *(ptr as *mut $strategy);
                <$strategy as nautilus_trading::Strategy>::core_mut(s)
            }
        }

        pub extern "C" fn strategy_external_order_claims(
            ptr: *const c_void,
            result: *mut Option<Vec<nautilus_model::identifiers::InstrumentId>>
        ) {
            unsafe {
                let s = &*(ptr as *const $strategy);
                *result = <$strategy as nautilus_trading::Strategy>::external_order_claims(s);
            }
        }

        pub extern "C" fn strategy_on_start(ptr: *mut c_void) -> i32 {
            unsafe {
                let s = &mut *(ptr as *mut $strategy);
                match nautilus_common::actor::DataActor::on_start(s) {
                    Ok(_) => 0,
                    Err(e) => {
                        log::error!("on_start error: {}", e);
                        -1
                    }
                }
            }
        }

        pub extern "C" fn strategy_on_stop(ptr: *mut c_void) -> i32 {
            unsafe {
                let s = &mut *(ptr as *mut $strategy);
                match nautilus_common::actor::DataActor::on_stop(s) {
                    Ok(_) => 0,
                    Err(e) => {
                        log::error!("on_stop error: {}", e);
                        -1
                    }
                }
            }
        }

        pub extern "C" fn strategy_on_time_event(
            ptr: *mut c_void,
            event: *const nautilus_common::timer::TimeEvent
        ) -> i32 {
            unsafe {
                let s = &mut *(ptr as *mut $strategy);
                match nautilus_common::actor::DataActor::on_time_event(s, &*event) {
                    Ok(_) => 0,
                    Err(_) => -1,
                }
            }
        }

        pub extern "C" fn strategy_on_instrument(
            ptr: *mut c_void,
            instrument: *const nautilus_model::instruments::InstrumentAny
        ) -> i32 {
            unsafe {
                let s = &mut *(ptr as *mut $strategy);
                match nautilus_common::actor::DataActor::on_instrument(s, &*instrument) {
                    Ok(_) => 0,
                    Err(_) => -1,
                }
            }
        }

        pub extern "C" fn strategy_on_book_deltas(
            ptr: *mut c_void,
            deltas: *const nautilus_model::data::OrderBookDeltas
        ) -> i32 {
            unsafe {
                let s = &mut *(ptr as *mut $strategy);
                match nautilus_common::actor::DataActor::on_book_deltas(s, &*deltas) {
                    Ok(_) => 0,
                    Err(_) => -1,
                }
            }
        }

        pub extern "C" fn strategy_on_book(
            ptr: *mut c_void,
            book: *const nautilus_model::orderbook::OrderBook
        ) -> i32 {
            unsafe {
                let s = &mut *(ptr as *mut $strategy);
                match nautilus_common::actor::DataActor::on_book(s, &*book) {
                    Ok(_) => 0,
                    Err(_) => -1,
                }
            }
        }

        pub extern "C" fn strategy_on_quote(
            ptr: *mut c_void,
            quote: *const nautilus_model::data::QuoteTick
        ) -> i32 {
            unsafe {
                let s = &mut *(ptr as *mut $strategy);
                match nautilus_common::actor::DataActor::on_quote(s, &*quote) {
                    Ok(_) => 0,
                    Err(_) => -1,
                }
            }
        }

        pub extern "C" fn strategy_on_trade(
            ptr: *mut c_void,
            trade: *const nautilus_model::data::TradeTick
        ) -> i32 {
            unsafe {
                let s = &mut *(ptr as *mut $strategy);
                match nautilus_common::actor::DataActor::on_trade(s, &*trade) {
                    Ok(_) => 0,
                    Err(_) => -1,
                }
            }
        }

        pub extern "C" fn strategy_on_bar(
            ptr: *mut c_void,
            bar: *const nautilus_model::data::Bar
        ) -> i32 {
            unsafe {
                let s = &mut *(ptr as *mut $strategy);
                match nautilus_common::actor::DataActor::on_bar(s, &*bar) {
                    Ok(_) => 0,
                    Err(_) => -1,
                }
            }
        }

        pub extern "C" fn strategy_on_mark_price(
            ptr: *mut c_void,
            mark_price: *const nautilus_model::data::MarkPriceUpdate
        ) -> i32 {
            unsafe {
                let s = &mut *(ptr as *mut $strategy);
                match nautilus_common::actor::DataActor::on_mark_price(s, &*mark_price) {
                    Ok(_) => 0,
                    Err(_) => -1,
                }
            }
        }

        pub extern "C" fn strategy_on_index_price(
            ptr: *mut c_void,
            index_price: *const nautilus_model::data::IndexPriceUpdate
        ) -> i32 {
            unsafe {
                let s = &mut *(ptr as *mut $strategy);
                match nautilus_common::actor::DataActor::on_index_price(s, &*index_price) {
                    Ok(_) => 0,
                    Err(_) => -1,
                }
            }
        }

        // ========== VTable 静态定义 ==========

        static STRATEGY_VTABLE: $crate::strategy_wrapper_ffi::StrategyVTable =
            $crate::strategy_wrapper_ffi::StrategyVTable {
                destroy: strategy_destroy,
                core: strategy_core,
                core_mut: strategy_core_mut,
                external_order_claims: strategy_external_order_claims,
                on_start: strategy_on_start,
                on_stop: strategy_on_stop,
                on_time_event: strategy_on_time_event,
                on_instrument: strategy_on_instrument,
                on_book_deltas: strategy_on_book_deltas,
                on_book: strategy_on_book,
                on_quote: strategy_on_quote,
                on_trade: strategy_on_trade,
                on_bar: strategy_on_bar,
                on_mark_price: strategy_on_mark_price,
                on_index_price: strategy_on_index_price,
            };

        // ========== 导出函数（需要 no_mangle）==========

        #[unsafe(no_mangle)]
        pub extern "C" fn get_vtable() -> &'static $crate::strategy_wrapper_ffi::StrategyVTable {
            &STRATEGY_VTABLE
        }

        #[unsafe(no_mangle)]
        pub extern "C" fn create_strategy(config_path: *const i8) -> *mut c_void {
            unsafe {
                let config_path = CStr::from_ptr(config_path)
                    .to_str()
                    .expect("Invalid config path");

                let config = <$config as $crate::SConfigSerializable>::from_json(config_path)
                    .expect("Failed to parse config");

                // 可选的配置处理
                $(
                    let config = $process_fn(config);
                )?

                let strategy = <$strategy>::new(config);
                Box::into_raw(Box::new(strategy)) as *mut c_void
            }
        }
    };
}
