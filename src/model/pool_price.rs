use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Result {
    pub price: String,
    pub sqrt_price: String,
    pub tick: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Params {
    pub subscription: String,
    pub result: Result,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PoolPrice {
    pub jsonrpc: String,
    pub method: String,
    pub params: Params,
}
