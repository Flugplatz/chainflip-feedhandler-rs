use std::sync::Arc;

use tokio::sync::{mpsc, oneshot, watch};

use crate::model::{asset_pair::AssetPair, liquidity::Liquidity, price_update::PriceUpdate};

/// Requests a `PoolInfoProviderHandle` can send to the `PoolInfoProvider` instance
pub enum PoolInfoProviderHandleMessage {
    SubscribePoolPriceUpdates {
        asset_pair: AssetPair,
    },
    GetLatestPoolPrice {
        asset_pair: AssetPair,
        tx: oneshot::Sender<Option<PriceUpdate>>,
    },
    GetStreamingPoolPriceUpdates {
        asset_pair: AssetPair,
        tx: oneshot::Sender<Option<watch::Receiver<Option<PriceUpdate>>>>,
    },
    GetLiquidity {
        asset_pair: AssetPair,
        tx: oneshot::Sender<Option<Liquidity>>,
    },
}

#[derive(Clone)]
pub struct PoolInfoProviderHandle {
    /// Sender channel for communicating with `PoolInfoProvider` instance
    pool_info_provider_handle_tx: Arc<mpsc::UnboundedSender<PoolInfoProviderHandleMessage>>,
}

impl PoolInfoProviderHandle {
    pub fn new(
        pool_info_provider_handle_tx: mpsc::UnboundedSender<PoolInfoProviderHandleMessage>,
    ) -> Self {
        PoolInfoProviderHandle {
            pool_info_provider_handle_tx: Arc::new(pool_info_provider_handle_tx),
        }
    }

    pub fn subscribe_pool_price_updates(&self, asset_pair: &AssetPair) {
        // TODO: dont consume result with `_`
        let _ = self.pool_info_provider_handle_tx.send(
            PoolInfoProviderHandleMessage::SubscribePoolPriceUpdates {
                asset_pair: asset_pair.clone(),
            },
        );
    }

    pub async fn get_streaming_pool_price_updates(
        &self,
        asset_pair: &AssetPair,
    ) -> Option<watch::Receiver<Option<PriceUpdate>>> {
        let (tx, rx) = oneshot::channel();

        let _ = self.pool_info_provider_handle_tx.send(
            PoolInfoProviderHandleMessage::GetStreamingPoolPriceUpdates {
                asset_pair: asset_pair.clone(),
                tx,
            },
        );

        rx.await.unwrap()
    }

    pub async fn get_latest_pool_price(&self, asset_pair: &AssetPair) -> Option<PriceUpdate> {
        let (tx, rx) = oneshot::channel();

        let _ = self.pool_info_provider_handle_tx.send(
            PoolInfoProviderHandleMessage::GetLatestPoolPrice {
                asset_pair: asset_pair.clone(),
                tx,
            },
        );

        rx.await.unwrap()
    }

    pub async fn get_pool_liquidity(&self, asset_pair: &AssetPair) -> Option<Liquidity> {
        let (tx, rx) = oneshot::channel();

        let _ =
            self.pool_info_provider_handle_tx
                .send(PoolInfoProviderHandleMessage::GetLiquidity {
                    asset_pair: asset_pair.clone(),
                    tx,
                });

        rx.await.unwrap()
    }
}
