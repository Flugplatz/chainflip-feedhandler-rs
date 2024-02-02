use lazy_static::lazy_static;
use primitive_types::U256;
use std::collections::HashMap;

use crate::model::{asset_pair::AssetPair, common::Tick};

lazy_static! {
    static ref DECIMALS: HashMap<String, u32> = [
        ("DOT".to_string(), 10),
        ("ETH".to_string(), 18),
        ("FLIP".to_string(), 18),
        ("BTC".to_string(), 8),
        ("USDC".to_string(), 6)
    ]
    .iter()
    .cloned()
    .collect();
}

/// Convert `Tick` into a floating point representaiton of price
pub fn tick_to_price(tick: Tick, asset_pair: &AssetPair) -> f64 {
    let decimals0 = *DECIMALS.get(&asset_pair.from).unwrap() as i32;
    let decimals1 = *DECIMALS.get(&asset_pair.to).unwrap() as i32;

    1.0001_f64.powi(tick) / 10_f64.powf((decimals1 - decimals0) as f64)
}

/// Convert hex string ie. "0xC0FFEE" into a `U256` decimal representation
pub fn hex_string_to_u256(hex_string: &str) -> U256 {
    let without_prefix = hex_string.trim_start_matches("0x");
    U256::from_str_radix(without_prefix, 16).unwrap()
}

#[cfg(test)]
mod tests {
    use float_cmp::approx_eq;
    use primitive_types::U256;

    use crate::{model::asset_pair::AssetPair, util::hex_string_to_u256};

    use super::tick_to_price;

    #[test]
    fn test_tick_to_price_btc() {
        let asset_pair = AssetPair {
            from: "BTC".to_string(),
            to: "USDC".to_string(),
        };

        let price: f64 = tick_to_price(57040, &asset_pair);
        assert!(approx_eq!(
            f64,
            29997.9703993,
            price,
            epsilon = 0.00000003,
            ulps = 2
        ));
    }

    #[test]
    fn test_tick_to_price_dot() {
        let asset_pair = AssetPair {
            from: "DOT".to_string(),
            to: "USDC".to_string(),
        };

        let price = tick_to_price(-69082, &asset_pair);
        assert!(approx_eq!(
            f64,
            9.99900670,
            price,
            epsilon = 0.00000003,
            ulps = 2
        ));
    }

    #[test]
    fn test_hex_string_to_u256() {
        assert_eq!(U256::from(1337), hex_string_to_u256("0x539"));
    }
}
