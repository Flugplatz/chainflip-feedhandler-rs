use serde::{Deserialize, Serialize};

use super::common::Tick;

#[derive(Debug, Serialize, Deserialize)]
pub struct LimitOrder {
    pub tick: Tick,
    pub amount: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LimitOrders {
    pub asks: Vec<LimitOrder>,
    pub bids: Vec<LimitOrder>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RangeOrder {
    pub tick: Tick,
    pub liquidity: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Result {
    pub limit_orders: LimitOrders,
    pub range_orders: Vec<RangeOrder>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Liquidity {
    pub id: String,
    pub jsonrpc: String,
    pub result: Result,
}
