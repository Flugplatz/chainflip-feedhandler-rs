# chainflip-feedhandler-rs
A feedhandler which comunicates with a chainflip node and monitors price updates and builds order books for multiple pools on a schedule.

## Usage
Run a testnet node via instructions here: https://github.com/chainflip-io/chainflip-perseverance  
You can inspect testnet information here: https://blocks-perseverance.chainflip.io/pools

```
CHAINFLIP_NODE_ADDR=192.168.1.70:9944 RUST_LOG=info cargo r
```

## Next Steps
* Decode `sqrt_price_x96` values.
* Implement order book functions which walk outwards to calculate slippage / volume weighted average price (VWAP).
    * Use Uniswap V3 maths to expand / contract liquidity as we traverse tick boundaries.
    * Limit orders should be consumed before range orders.
    * Fee estimation.