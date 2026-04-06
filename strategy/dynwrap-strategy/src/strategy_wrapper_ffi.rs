use anyhow::{Context, Result};
use libloading::{Library, Symbol};
use log::info;
use nautilus_common::actor::{DataActor, DataActorCore};
use nautilus_common::timer::TimeEvent;
use nautilus_model::data::{
    Bar, IndexPriceUpdate, MarkPriceUpdate, OrderBookDeltas, QuoteTick, TradeTick,
};
use nautilus_model::identifiers::InstrumentId;
use nautilus_model::instruments::InstrumentAny;
use nautilus_model::orderbook::OrderBook;
use nautilus_trading::{Strategy, StrategyCore};
use std::ffi::{CStr, CString};
use std::fmt;
use std::ops::{Deref, DerefMut};
use std::os::raw::{c_char, c_int, c_void};
use std::path::Path;

// ============================================================================
// VTable 定义
// ============================================================================

#[repr(C)]
#[derive(Copy, Clone)]
pub struct StrategyVTable {
    // === 生命周期 ===
    pub destroy: extern "C" fn(*mut c_void),

    // === Strategy trait ===
    pub core: extern "C" fn(*const c_void) -> *const StrategyCore,
    pub core_mut: extern "C" fn(*mut c_void) -> *mut StrategyCore,
    pub external_order_claims: extern "C" fn(*const c_void, *mut Option<Vec<InstrumentId>>),

    // === DataActor trait ===
    pub on_start: extern "C" fn(*mut c_void) -> i32,
    pub on_stop: extern "C" fn(*mut c_void) -> i32,
    pub on_time_event: extern "C" fn(*mut c_void, *const TimeEvent) -> i32,
    pub on_instrument: extern "C" fn(*mut c_void, *const InstrumentAny) -> i32,
    pub on_book_deltas: extern "C" fn(*mut c_void, *const OrderBookDeltas) -> i32,
    pub on_book: extern "C" fn(*mut c_void, *const OrderBook) -> i32,
    pub on_quote: extern "C" fn(*mut c_void, *const QuoteTick) -> i32,
    pub on_trade: extern "C" fn(*mut c_void, *const TradeTick) -> i32,
    pub on_bar: extern "C" fn(*mut c_void, *const Bar) -> i32,
    pub on_mark_price: extern "C" fn(*mut c_void, *const MarkPriceUpdate) -> i32,
    pub on_index_price: extern "C" fn(*mut c_void, *const IndexPriceUpdate) -> i32,
}

impl fmt::Debug for StrategyVTable {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("StrategyVTable")
            .field("destroy", &format_args!("0x{:016x}", self.destroy as usize))
            .field("core", &format_args!("0x{:016x}", self.core as usize))
            .field(
                "core_mut",
                &format_args!("0x{:016x}", self.core_mut as usize),
            )
            .field(
                "on_start",
                &format_args!("0x{:016x}", self.on_start as usize),
            )
            .field("on_bar", &format_args!("0x{:016x}", self.on_bar as usize))
            .finish_non_exhaustive()
    }
}

// ============================================================================
// 包装器
// ============================================================================

pub struct DynStrategyWrapper {
    ptr: *mut c_void,
    vtable: StrategyVTable,
    #[allow(dead_code)]
    library: Library,
}

impl DynStrategyWrapper {
    /// 从动态库加载策略
    pub fn load(lib_path: &Path, config_path: &str) -> Result<Self> {
        info!("Loading strategy from: {:?}", lib_path);
        info!("Config path: {}", config_path);

        unsafe {
            // 加载动态库
            let library = Library::new(lib_path).context("Failed to load dynamic library")?;

            // 获取创建函数
            let create_strategy: Symbol<extern "C" fn(*const c_char) -> *mut c_void> = library
                .get(b"create_strategy")
                .context("Failed to get create_strategy symbol")?;

            // 获取 vtable 函数
            let get_vtable: Symbol<extern "C" fn() -> &'static StrategyVTable> = library
                .get(b"get_vtable")
                .context("Failed to get get_vtable symbol")?;

            // 转换配置路径为 C 字符串
            let config_path_cstr =
                CString::new(config_path).context("Config path contains null byte")?;

            // 创建策略实例
            let ptr = create_strategy(config_path_cstr.as_ptr() as *const c_char);
            if ptr.is_null() {
                anyhow::bail!("create_strategy returned null");
            }

            // 获取 vtable
            let vtable = get_vtable();

            info!("Strategy loaded successfully, ptr: {:?}", ptr);

            Ok(Self {
                ptr,
                vtable: *vtable,
                library,
            })
        }
    }

    /// 获取策略指针（用于调试）
    pub fn ptr(&self) -> *mut c_void {
        self.ptr
    }

    /// 获取 vtable 引用（用于调试）
    pub fn vtable(&self) -> &StrategyVTable {
        &self.vtable
    }
}

impl fmt::Debug for DynStrategyWrapper {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("DynStrategyWrapper")
            .field("ptr", &self.ptr)
            .field("vtable", &self.vtable)
            .finish()
    }
}

impl Drop for DynStrategyWrapper {
    fn drop(&mut self) {
        info!("Destroying DynStrategyWrapper");
        unsafe {
            (self.vtable.destroy)(self.ptr);
        }
    }
}

// ============================================================================
// Deref 实现
// ============================================================================

impl Deref for DynStrategyWrapper {
    type Target = DataActorCore;

    fn deref(&self) -> &Self::Target {
        unsafe {
            let core_ptr = (self.vtable.core)(self.ptr);
            core_ptr.as_ref().expect("core returned null")
        }
    }
}

impl DerefMut for DynStrategyWrapper {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe {
            let core_ptr = (self.vtable.core_mut)(self.ptr);
            core_ptr.as_mut().expect("core_mut returned null")
        }
    }
}

// ============================================================================
// Strategy trait 实现
// ============================================================================

impl Strategy for DynStrategyWrapper {
    fn core(&self) -> &StrategyCore {
        unsafe {
            let core_ptr = (self.vtable.core)(self.ptr);
            core_ptr.as_ref().expect("core returned null")
        }
    }

    fn core_mut(&mut self) -> &mut StrategyCore {
        unsafe {
            let core_ptr = (self.vtable.core_mut)(self.ptr);
            core_ptr.as_mut().expect("core_mut returned null")
        }
    }

    fn external_order_claims(&self) -> Option<Vec<InstrumentId>> {
        let mut result = None;
        unsafe {
            (self.vtable.external_order_claims)(self.ptr, &mut result as *mut _);
        }
        result
    }
}

// ============================================================================
// DataActor trait 实现
// ============================================================================

impl DataActor for DynStrategyWrapper {
    fn on_start(&mut self) -> Result<()> {
        let ret = unsafe { (self.vtable.on_start)(self.ptr) };
        if ret == 0 {
            Ok(())
        } else {
            Err(anyhow::anyhow!("on_start failed with code {}", ret))
        }
    }

    fn on_stop(&mut self) -> Result<()> {
        let ret = unsafe { (self.vtable.on_stop)(self.ptr) };
        if ret == 0 {
            Ok(())
        } else {
            Err(anyhow::anyhow!("on_stop failed with code {}", ret))
        }
    }

    fn on_time_event(&mut self, event: &TimeEvent) -> Result<()> {
        let ret = unsafe { (self.vtable.on_time_event)(self.ptr, event as *const _) };
        if ret == 0 {
            Ok(())
        } else {
            Err(anyhow::anyhow!("on_time_event failed"))
        }
    }

    fn on_instrument(&mut self, instrument: &InstrumentAny) -> Result<()> {
        let ret = unsafe { (self.vtable.on_instrument)(self.ptr, instrument as *const _) };
        if ret == 0 {
            Ok(())
        } else {
            Err(anyhow::anyhow!("on_instrument failed"))
        }
    }

    fn on_book_deltas(&mut self, deltas: &OrderBookDeltas) -> Result<()> {
        let ret = unsafe { (self.vtable.on_book_deltas)(self.ptr, deltas as *const _) };
        if ret == 0 {
            Ok(())
        } else {
            Err(anyhow::anyhow!("on_book_deltas failed"))
        }
    }

    fn on_book(&mut self, book: &OrderBook) -> Result<()> {
        let ret = unsafe { (self.vtable.on_book)(self.ptr, book as *const _) };
        if ret == 0 {
            Ok(())
        } else {
            Err(anyhow::anyhow!("on_book failed"))
        }
    }

    fn on_quote(&mut self, quote: &QuoteTick) -> Result<()> {
        let ret = unsafe { (self.vtable.on_quote)(self.ptr, quote as *const _) };
        if ret == 0 {
            Ok(())
        } else {
            Err(anyhow::anyhow!("on_quote failed"))
        }
    }

    fn on_trade(&mut self, trade: &TradeTick) -> Result<()> {
        let ret = unsafe { (self.vtable.on_trade)(self.ptr, trade as *const _) };
        if ret == 0 {
            Ok(())
        } else {
            Err(anyhow::anyhow!("on_trade failed"))
        }
    }

    fn on_bar(&mut self, bar: &Bar) -> Result<()> {
        let ret = unsafe { (self.vtable.on_bar)(self.ptr, bar as *const _) };
        if ret == 0 {
            Ok(())
        } else {
            Err(anyhow::anyhow!("on_bar failed"))
        }
    }

    fn on_mark_price(&mut self, mark_price: &MarkPriceUpdate) -> Result<()> {
        let ret = unsafe { (self.vtable.on_mark_price)(self.ptr, mark_price as *const _) };
        if ret == 0 {
            Ok(())
        } else {
            Err(anyhow::anyhow!("on_mark_price failed"))
        }
    }

    fn on_index_price(&mut self, index_price: &IndexPriceUpdate) -> Result<()> {
        let ret = (self.vtable.on_index_price)(self.ptr, index_price as *const _);
        if ret == 0 {
            Ok(())
        } else {
            Err(anyhow::anyhow!("on_index_price failed"))
        }
    }
}
