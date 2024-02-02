use super::{asset_pair::AssetPair, common::Tick};

#[derive(Clone, Debug)]
pub struct PriceUpdate {
    pub asset_pair: AssetPair,
    pub price: String,
    pub sqrt_price: String,
    pub tick: Tick,
}
