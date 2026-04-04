// -------------------------------------------------------------------------------------------------
//  Copyright (C) 2015-2026 Nautech Systems Pty Ltd. All rights reserved.
//  https://nautechsystems.io
//
//  Licensed under the GNU Lesser General Public License Version 3.0 (the "License");
//  You may not use this file except in compliance with the License.
//  You may obtain a copy of the License at https://www.gnu.org/licenses/lgpl-3.0.en.html
//
//  Unless required by applicable law or agreed to in writing, software
//  distributed under the License is distributed on an "AS IS" BASIS,
//  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//  See the License for the specific language governing permissions and
//  limitations under the License.
// -------------------------------------------------------------------------------------------------

//! Example demonstrating live execution testing with the OKX adapter.
//!
//! Run with: `cargo run --example okx-exec-tester --package nautilus-okx`

use dynwrap_strategy::SConfigSerializable;
use mq_demo_strategy::config::ExecTesterConfig;
use nautilus_model::{
    identifiers::{ClientId, InstrumentId, StrategyId},
    types::Quantity,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client_id = ClientId::new("OKX");
    let instrument_id = InstrumentId::from("SOL-USDT-SWAP.OKX");

    let mut tester_config = ExecTesterConfig::new(
        StrategyId::from("EXEC_TESTER-001"),
        instrument_id,
        client_id,
        Quantity::from("0.01"),
    )
    .with_log_data(false)
    // .with_enable_limit_buys(false)
    // .with_enable_limit_sells(false)
    // .with_enable_stop_sells(true)
    // .with_stop_order_type(OrderType::TrailingStopMarket)
    // .with_trailing_offset(Decimal::from(100))
    // .with_trailing_offset_type(TrailingOffsetType::BasisPoints)
    // .with_stop_offset_ticks(50)
    .with_cancel_orders_on_stop(true)
    .with_close_positions_on_stop(true);

    tester_config.base.external_order_claims = Some(vec![instrument_id]);

    // Use UUIDs for unique client order IDs across restarts
    tester_config.base.use_uuid_client_order_ids = true;
    // OKX doesn't allow hyphens in client order IDs
    tester_config.base.use_hyphens_in_client_order_ids = false;
    tester_config.write_to_json().unwrap();

    Ok(())
}
