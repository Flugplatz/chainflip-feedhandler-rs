use std::{env, time::Duration};

use orderbook_builder::create_and_start_order_book_builder;
use pool_info_provider::pool_info_provider::PoolInfoProvider;
use tokio::time::sleep;

use crate::model::asset_pair::AssetPair;
use simple_logger::SimpleLogger;
mod model;
mod orderbook_builder;
mod pool_info_provider;
mod util;

mod constants {
    use std::time::Duration;

    pub const ORDERBOOK_POLL_DURATION: Duration = Duration::from_secs(15);
}

#[tokio::main]
async fn main() {
    SimpleLogger::new().env().init().unwrap();

    let node_address = match env::var("CHAINFLIP_NODE_ADDR") {
        Ok(addr) => addr,
        Err(e) => panic!(
            "No chainflip node address (CHAINFLIP_NODE_ADDR) set: {:?}",
            e
        ),
    };

    let pools = vec![
        AssetPair::new("BTC".to_string(), "USDC".to_string()),
        AssetPair::new("FLIP".to_string(), "USDC".to_string()),
        AssetPair::new("DOT".to_string(), "USDC".to_string()),
        AssetPair::new("ETH".to_string(), "USDC".to_string()),
    ];

    // create and start the pool info provider and subscribe to price updates on each pool
    let pool_provider_handle = {
        let mut pool_info_provider = PoolInfoProvider::new(&node_address);
        let handle = pool_info_provider.get_handle();

        tokio::spawn(async move {
            pool_info_provider.run().await;
        });

        handle
    };

    for pool in pools.iter() {
        pool_provider_handle.subscribe_pool_price_updates(pool);
    }

    // FIXME: hack to wait for subscriptions to be setup
    sleep(Duration::from_secs(5)).await;

    // get a price_update handle for FLIP-USDC
    let mut price_update_rx = pool_provider_handle
        .get_streaming_pool_price_updates(&pools[1])
        .await
        .unwrap();

    // create and start an order book builder for BTC-USDC
    let mut btc_orderbook_rx = create_and_start_order_book_builder(
        &pools[0],
        pool_provider_handle.clone(),
        constants::ORDERBOOK_POLL_DURATION,
    );

    // get the latest price for flip
    let price_update = pool_provider_handle
        .get_latest_pool_price(&pools[1])
        .await
        .unwrap();
    log::info!("FLIP-USC price update: {:?}", price_update);

    // listen for different types of updates on the channels were interested in
    loop {
        tokio::select! {
            ob = btc_orderbook_rx.recv() => {
                let ob = match ob {
                    Some(ob) => ob,
                    None => {
                        log::error!("error receiving orderbook");

                        break;
                    },
                };

                log::info!("Received orderbook: {:?}", ob);
            },
            _ = price_update_rx.changed() => {
                match price_update_rx.borrow_and_update().as_ref() {
                    Some(pu) => {
                        log::info!("Received price update: {:?}", pu);
                    },
                    None => {
                        log::error!("error receiving price update");

                        break;
                    },
                }
            }
        }
    }
}
