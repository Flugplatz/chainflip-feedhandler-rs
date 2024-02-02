use std::time::Duration;
use tokio::sync::mpsc;

use tokio::time::{interval, sleep, Interval};

use crate::model::asset_pair::AssetPair;
use crate::model::order_book::OrderBook;
use crate::pool_info_provider::pool_info_provider_handle::PoolInfoProviderHandle;
use crate::util::hex_string_to_u256;

/// An enduring thread which queries liquidity information from a `PoolInfoProviderHandle` periodically
/// to build an `OrderBook` before sending down stream over a channel.
///
/// The trigger for building an order book is a combination of time based (`poll_duration`) and we also
/// watch for price change updates and use this as an additional trigger to build books.
pub struct OrderBookBuilder {
    /// Asset pair of interest
    asset_pair: AssetPair,
    /// Handle for making REST calls
    pool_info_provider_handle: PoolInfoProviderHandle,
    /// Poll duration between fetching & building books
    poll_duration: Duration,
    /// Downstream channel for consumers
    book_sender: mpsc::UnboundedSender<OrderBook>,
}

/// Create and start an order book builder and return the channel it publishes updates on
pub fn create_and_start_order_book_builder(
    asset_pair: &AssetPair,
    pool_info_provider_handle: PoolInfoProviderHandle,
    poll_duration: Duration,
) -> mpsc::UnboundedReceiver<OrderBook> {
    let (tx, rx) = mpsc::unbounded_channel();
    let orderbook_builder = OrderBookBuilder::new(
        asset_pair.clone(),
        pool_info_provider_handle,
        poll_duration,
        tx,
    );

    tokio::spawn(async move {
        orderbook_builder.run().await;
    });

    rx
}

impl OrderBookBuilder {
    pub fn new(
        asset_pair: AssetPair,
        pool_info_provider_handle: PoolInfoProviderHandle,
        poll_duration: Duration,
        book_sender: mpsc::UnboundedSender<OrderBook>,
    ) -> Self {
        OrderBookBuilder {
            asset_pair,
            pool_info_provider_handle,
            poll_duration,
            book_sender,
        }
    }

    pub async fn run(&self) {
        let mut update_interval: Interval = interval(self.poll_duration);

        // poll until a price update watch channel is available for this AssetPair
        let mut price_update_watch = loop {
            match self
                .pool_info_provider_handle
                .get_streaming_pool_price_updates(&self.asset_pair)
                .await
            {
                Some(watch) => {
                    break watch;
                }
                None => {
                    sleep(Duration::from_secs(5)).await;
                }
            }
        };

        let mut latest_pool_price = price_update_watch.borrow().as_ref().unwrap().clone();

        loop {
            // block until the orderbook update interval has elapsed or a price update occurs
            tokio::select! {
                _ = update_interval.tick() => {},
                _ = price_update_watch.changed() => {
                    update_interval.reset_immediately();

                    latest_pool_price = price_update_watch.borrow().as_ref().unwrap().clone();
                }
            };

            let liquidity = self
                .pool_info_provider_handle
                .get_pool_liquidity(&self.asset_pair)
                .await
                .unwrap();

            // build order book
            let sqrt_price_x96 = hex_string_to_u256(&latest_pool_price.sqrt_price);
            let ob = OrderBook::new(
                &self.asset_pair,
                liquidity,
                sqrt_price_x96,
                latest_pool_price.tick,
            );

            // send order book to consumers
            match self.book_sender.send(ob) {
                Ok(_) => {}
                Err(e) => {
                    log::error!("error sending orderbook update: {:?}", e);

                    break;
                }
            }
        }
    }
}
