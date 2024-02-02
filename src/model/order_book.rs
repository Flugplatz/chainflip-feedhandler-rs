use crate::util::{hex_string_to_u256, tick_to_price};

use super::{
    asset_pair::AssetPair,
    common::{Amount, SqrtPriceQ64F96, Tick},
    liquidity::Liquidity,
};

#[derive(Debug)]
enum Side {
    Buy,
    Sell,
}

#[derive(Debug)]
pub struct LimitOrder {
    #[allow(unused)]
    side: Side,
    #[allow(unused)]
    tick: Tick,
    #[allow(unused)]
    amount: Amount,
}

#[derive(Debug)]
pub struct RangeOrder {
    #[allow(unused)]
    start_tick: Tick,
    #[allow(unused)]
    end_tick: Tick,
    #[allow(unused)]
    liquidity: Amount,
}

#[derive(Debug)]
pub struct OrderBook {
    pub asset_pair: AssetPair,
    pub sqrt_price_x96: SqrtPriceQ64F96,
    pub tick: Tick,
    pub tick_price: f64,
    pub limit_bids: Vec<LimitOrder>,
    pub limit_asks: Vec<LimitOrder>,
    pub range_orders: Vec<RangeOrder>,
}

impl OrderBook {
    pub fn new(
        asset_pair: &AssetPair,
        liquidity: Liquidity,
        sqrt_price_x96: SqrtPriceQ64F96,
        tick: Tick,
    ) -> Self {
        let tick_price = tick_to_price(tick, asset_pair);

        let limit_bids: Vec<LimitOrder> = liquidity
            .result
            .limit_orders
            .bids
            .iter()
            .map(|b| LimitOrder {
                side: Side::Buy,
                tick: b.tick,
                amount: hex_string_to_u256(&b.amount),
            })
            .collect();

        let limit_asks: Vec<LimitOrder> = liquidity
            .result
            .limit_orders
            .asks
            .iter()
            .map(|a| LimitOrder {
                side: Side::Sell,
                tick: a.tick,
                amount: hex_string_to_u256(&a.amount),
            })
            .collect();

        let range_orders = liquidity.result.range_orders;
        let zipped_it = range_orders.iter().zip(range_orders.iter().skip(1));
        let range_orders: Vec<RangeOrder> = zipped_it
            .map(|(range_start, range_end)| RangeOrder {
                start_tick: range_start.tick,
                end_tick: range_end.tick,
                liquidity: hex_string_to_u256(&range_start.liquidity),
            })
            .collect();

        OrderBook {
            asset_pair: asset_pair.clone(),
            sqrt_price_x96,
            tick,
            tick_price,
            limit_bids,
            limit_asks,
            range_orders,
        }
    }
}

#[cfg(test)]
mod tests {
    use primitive_types::U256;

    use crate::model::{
        asset_pair::AssetPair,
        common::{SqrtPriceQ64F96, Tick},
        liquidity::{LimitOrder, LimitOrders, Liquidity, RangeOrder, Result},
    };

    use super::OrderBook;

    #[test]
    fn test_new_orderbook() {
        let liquidity = Liquidity {
            id: "1".to_string(),
            jsonrpc: "2".to_string(),
            result: Result {
                limit_orders: LimitOrders {
                    asks: vec![LimitOrder {
                        tick: 1234,
                        amount: "0x01".to_string(),
                    }],
                    bids: vec![LimitOrder {
                        tick: 1233,
                        amount: "0x01".to_string(),
                    }],
                },
                range_orders: vec![
                    RangeOrder {
                        tick: -1,
                        liquidity: "0x01".to_string(),
                    },
                    RangeOrder {
                        tick: 10,
                        liquidity: "0x01".to_string(),
                    },
                    RangeOrder {
                        tick: 100,
                        liquidity: "0x0".to_string(),
                    },
                ],
            },
        };

        let asset_pair = AssetPair {
            from: "BTC".to_string(),
            to: "USDC".to_string(),
        };
        let sqrt_price_x96: SqrtPriceQ64F96 = U256::zero();
        let tick: Tick = 1234;

        let ob = OrderBook::new(&asset_pair, liquidity, sqrt_price_x96, tick);
        assert_eq!(1, ob.limit_asks.len());
        assert_eq!(1, ob.limit_bids.len());

        assert_eq!(2, ob.range_orders.len());

        let range_order_0 = ob.range_orders.get(0).unwrap();
        assert_eq!(-1, range_order_0.start_tick);
        assert_eq!(10, range_order_0.end_tick);

        let range_order_1 = ob.range_orders.get(1).unwrap();
        assert_eq!(10, range_order_1.start_tick);
        assert_eq!(100, range_order_1.end_tick);
    }
}
