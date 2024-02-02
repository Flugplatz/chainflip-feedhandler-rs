use std::collections::HashMap;

use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::sync::{mpsc, watch};
use tokio_tungstenite::{connect_async, tungstenite::Message};

use crate::model::{
    asset_pair::AssetPair, json_rpc::JsonRpcResponse, pool_price::PoolPrice,
    price_update::PriceUpdate,
};

use super::pool_info_provider_handle::{PoolInfoProviderHandle, PoolInfoProviderHandleMessage};
use rand::prelude::*;

/// Map of AssetPair to tokio watch channel (tx, rx)
type AssetWatchChannelMap = HashMap<
    AssetPair,
    (
        watch::Sender<Option<PriceUpdate>>,
        watch::Receiver<Option<PriceUpdate>>,
    ),
>;

/// Websocket messages supported
#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
enum WebsocketMessage {
    /// Pool price update from cf_subscribe_pool_price
    PoolPrice(PoolPrice),
    /// Response to a cf_subscribe_pool_price message with subscription id
    JsonRpcResponse(JsonRpcResponse),
}

/// An enduring thread which owns both websocket and REST communications with the node.
///
/// A single websocket is opened and multiple subscriptions to `cf_subscribe_pool_price` for different
/// asset pairs can be made. Updates are pushed downstream internally (per asset pair) via a tokio::watch channel.
pub struct PoolInfoProvider {
    /// Hostname of node
    hostname: String,
    /// Map of request_id to corresponding asset pair
    request_id_map: HashMap<String, AssetPair>,
    /// Map of asset pair to tokio::watch channels for sending updates downstream
    asset_watch_channel_map: AssetWatchChannelMap,
    /// Map of subscription id to asset_pair for attributing websocket messages
    /// to relevant asset pair.
    subscription_map: HashMap<String, AssetPair>,
    /// Handle which internal clients use to issue requests to this struct
    handle: PoolInfoProviderHandle,
    /// internal channel over which we receive requests from client handles
    internal_rx: mpsc::UnboundedReceiver<PoolInfoProviderHandleMessage>,
}

impl PoolInfoProvider {
    /// Create a new instance of `PoolInfoProvider`
    pub fn new(hostname: &str) -> Self {
        let (internal_tx, internal_rx) = mpsc::unbounded_channel();
        let handle = PoolInfoProviderHandle::new(internal_tx);

        PoolInfoProvider {
            hostname: hostname.to_string(),
            request_id_map: HashMap::new(),
            asset_watch_channel_map: HashMap::new(),
            subscription_map: HashMap::new(),
            handle,
            internal_rx,
        }
    }

    /// Get a clone of the `PoolInfoProviderHandle` for interacting with this instance
    pub fn get_handle(&self) -> PoolInfoProviderHandle {
        self.handle.clone()
    }

    /// Enduring loop, process websocket message and internal requests
    pub async fn run(&mut self) {
        let (ws_stream, _) = connect_async(format!("ws://{}", &self.hostname))
            .await
            .expect("error connecting to websocket");
        let (mut ws_write, mut ws_read) = ws_stream.split();

        loop {
            tokio::select! {
                websocket_message = ws_read.next() => {
                    let websocket_message = match websocket_message {
                        Some(msg) => match msg {
                            Ok(msg) => match msg {
                                Message::Text(msg) => msg,
                                _ => {

                                    continue;
                                }
                            },
                            Err(e) => {
                                log::error!("error receiving websocket message: {:?}", e);

                                break;
                            },
                        },
                        None => {
                            log::error!("websocket disconnected");

                            break;
                        }
                    };

                    log::trace!("websocket recv: {:?}", &websocket_message);

                    let deser: WebsocketMessage = serde_json::from_str(&websocket_message).unwrap();
                    match deser {
                        WebsocketMessage::PoolPrice(pp) => {
                            let asset_pair = self.subscription_map.get(&pp.params.subscription).unwrap();
                            let update = PriceUpdate {asset_pair:asset_pair.clone(),price:pp.params.result.price,sqrt_price:pp.params.result.sqrt_price,tick:pp.params.result.tick };

                            let (tx, _) = self.asset_watch_channel_map.get(asset_pair).unwrap();

                            let send_result = tx.send(Some(update));

                            match send_result {
                                Ok(_) => {},
                                Err(e) => {
                                    log::error!("error sending latest price update: {:?}", e);

                                    break;
                                },
                            }
                        },
                        WebsocketMessage::JsonRpcResponse(resp) => {
                            let asset_pair = self.request_id_map.get(&resp.id).unwrap();
                            self.subscription_map.insert(resp.result, asset_pair.clone());
                        },
                    }
                },
                internal_message = self.internal_rx.recv() => {
                    match internal_message {
                        Some(msg) => {
                            match msg {
                                PoolInfoProviderHandleMessage::SubscribePoolPriceUpdates { asset_pair } => {
                                    if self.asset_watch_channel_map.contains_key(&asset_pair) {

                                        continue;
                                    }

                                    let (tx, rx) = watch::channel(None);
                                    self.asset_watch_channel_map.insert(asset_pair.clone(), (tx, rx));

                                    let request_id = {
                                        let mut rng = rand::thread_rng();
                                        rng.gen::<i32>()
                                    }.to_string();

                                    self.request_id_map.insert(request_id.clone(), asset_pair.clone());

                                    let to_send = json!({
                                        "jsonrpc": "2.0",
                                        "id": request_id,
                                        "method": "cf_subscribe_pool_price",
                                        "params": {
                                            "from_asset": format!("{}", asset_pair.from),
                                            "to_asset": format!("{}", asset_pair.to),
                                        }
                                    }).to_string();

                                    log::info!("subscribing to cf_subscribe_pool_price for {:?}", &asset_pair);

                                    match ws_write.send(Message::Text(to_send)).await {
                                        Ok(_) => {},
                                        Err(e) => {
                                            log::error!("error writing to websocket: {:?}", e);

                                            break;
                                        },
                                    }
                                },
                                PoolInfoProviderHandleMessage::GetLatestPoolPrice { asset_pair, tx } => {
                                    let response = match self.asset_watch_channel_map.get(&asset_pair) {
                                        Some((_, rx)) => {
                                            rx.borrow().clone()
                                        },
                                        None => None
                                    };

                                    match tx.send(response) {
                                        Ok(_) => {},
                                        Err(e) => {
                                            log::error!("error sending GetLatestPoolPrice client response: {:?}", e);

                                            break;
                                        },
                                    }
                                },
                                PoolInfoProviderHandleMessage::GetStreamingPoolPriceUpdates { asset_pair, tx } => {
                                    let response = self.asset_watch_channel_map.get(&asset_pair).map(|(_, rx)| rx.clone());

                                    match tx.send(response) {
                                        Ok(_) => {},
                                        Err(e) => {
                                            log::error!("error sending GetStreamingPoolPriceUpdates client response: {:?}", e);

                                            break;
                                        },
                                    }
                                },
                                PoolInfoProviderHandleMessage::GetLiquidity { asset_pair, tx } => {
                                    let client = reqwest::Client::new();

                                    let to_send = json!({
                                        "jsonrpc": "2.0",
                                        "id": "1",
                                        "method": "cf_pool_liquidity",
                                        "params": {
                                            "base_asset": format!("{}", asset_pair.from),
                                            "quote_asset": format!("{}", asset_pair.to),
                                        }
                                    });

                                    let resp = client
                                        .post(format!("http://{}", &self.hostname))
                                        .json(&to_send)
                                        .send()
                                        .await
                                        .unwrap();

                                    let response_text = resp.text().await.unwrap();
                                    let liquidity = serde_json::from_str(&response_text).unwrap();

                                    match tx.send(Some(liquidity)) {
                                        Ok(_) => {},
                                        Err(e) => {
                                            log::error!("error sending GetLiquidity client response: {:?}", e);

                                            break;
                                        },
                                    }
                                },
                            }
                        },
                        None => {
                            log::error!("error receiving internal message");

                            break;
                        },
                    }
                }
            }
        }
    }
}
