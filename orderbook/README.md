# Order matching engine (orderbook)

Project is just a basic order-matching engine (orderbook), created especially for learning Rust and internals of trading systems. It features a sequential orderbook and an orderbook that uses global unique IDs. The sequential orderbook was borrowed from [here](https://github.com/dgtony/orderbook-rs). The primary improvements that the guid version offers is the use of BigDecimal for the asset amounts and the use of UUIDs for the order ID. 

Each instance of orderbook is a single-threaded reactive module for the certain currency pair. It consumes orders and return vector of events, generated during processing.

Supported features:

* market orders
* limit orders
* amending limit order price/quantity
* cancelling limit order
* partial filling


## Usage
Full example code could be found in `bin/example.rs`. Here is event log created in processing test orders via `cargo run bin/example.rs`

```
Order => NewLimitOrder { order_id: eca3f382-9b57-4529-9e5a-409012fe123e, order_asset: BTC, price_asset: USD, side: Bid, price: BigDecimal("41711.760112"), qty: BigDecimal("0.15"), ts: SystemTime { tv_sec: 1695144728, tv_nsec: 283286000 } }
Processing => [Ok(Accepted { order_id: eca3f382-9b57-4529-9e5a-409012fe123e, order_asset: BTC, order_type: Limit, price_asset: USD, price: Some(BigDecimal("41711.760112")), qty: BigDecimal("0.15"), side: Bid, ts: SystemTime { tv_sec: 1695144728, tv_nsec: 283408000 } })]
Spread => not available

Order => NewLimitOrder { order_id: b7acaed6-4036-4cba-a81d-a975f1142e4f, order_asset: BTC, price_asset: USD, side: Ask, price: BigDecimal("41712.60777901"), qty: BigDecimal("1.0223"), ts: SystemTime { tv_sec: 1695144728, tv_nsec: 283295000 } }
Processing => [Ok(Accepted { order_id: b7acaed6-4036-4cba-a81d-a975f1142e4f, order_asset: BTC, order_type: Limit, price_asset: USD, price: Some(BigDecimal("41712.60777901")), qty: BigDecimal("1.0223"), side: Ask, ts: SystemTime { tv_sec: 1695144728, tv_nsec: 283424000 } })]
Spread => bid: 41711.760112, ask: 41712.60777901

Order => NewLimitOrder { order_id: 2187ac07-8f7c-4e4f-8b9f-9ab1a0fb7364, order_asset: BTC, price_asset: USD, side: Bid, price: BigDecimal("1.01"), qty: BigDecimal("0.4"), ts: SystemTime { tv_sec: 1695144728, tv_nsec: 283299000 } }
Processing => [Ok(Accepted { order_id: 2187ac07-8f7c-4e4f-8b9f-9ab1a0fb7364, order_asset: BTC, order_type: Limit, price_asset: USD, price: Some(BigDecimal("1.01")), qty: BigDecimal("0.4"), side: Bid, ts: SystemTime { tv_sec: 1695144728, tv_nsec: 283443000 } })]
Spread => bid: 41711.760112, ask: 41712.60777901

Order => NewLimitOrder { order_id: da54fcb4-602c-4f27-9606-cf3d572b1ed3, order_asset: BTC, price_asset: USD, side: Ask, price: BigDecimal("1.03"), qty: BigDecimal("0.5"), ts: SystemTime { tv_sec: 1695144728, tv_nsec: 283302000 } }
Processing => [Ok(Accepted { order_id: da54fcb4-602c-4f27-9606-cf3d572b1ed3, order_asset: BTC, order_type: Limit, price_asset: USD, price: Some(BigDecimal("1.03")), qty: BigDecimal("0.5"), side: Ask, ts: SystemTime { tv_sec: 1695144728, tv_nsec: 283460000 } }), Ok(PartiallyFilled { order_id: da54fcb4-602c-4f27-9606-cf3d572b1ed3, side: Ask, order_type: Limit, price: BigDecimal("41711.760112"), qty: BigDecimal("0.15"), ts: SystemTime { tv_sec: 1695144728, tv_nsec: 283462000 } }), Ok(Filled { order_id: eca3f382-9b57-4529-9e5a-409012fe123e, side: Bid, order_type: Limit, price: BigDecimal("41711.760112"), qty: BigDecimal("0.15"), ts: SystemTime { tv_sec: 1695144728, tv_nsec: 283462000 } })]
Spread => bid: 1.01, ask: 1.03

Order => NewMarketOrder { order_id: 315c9c8a-15d8-411f-8683-24c76090af10, order_asset: BTC, price_asset: USD, side: Bid, qty: BigDecimal("0.90"), ts: SystemTime { tv_sec: 1695144728, tv_nsec: 283304000 } }
Processing => [Ok(Accepted { order_id: 315c9c8a-15d8-411f-8683-24c76090af10, order_asset: BTC, order_type: Market, price_asset: USD, price: None, qty: BigDecimal("0.90"), side: Bid, ts: SystemTime { tv_sec: 1695144728, tv_nsec: 283487000 } }), Ok(PartiallyFilled { order_id: 315c9c8a-15d8-411f-8683-24c76090af10, side: Bid, order_type: Market, price: BigDecimal("1.03"), qty: BigDecimal("0.35"), ts: SystemTime { tv_sec: 1695144728, tv_nsec: 283489000 } }), Ok(Filled { order_id: da54fcb4-602c-4f27-9606-cf3d572b1ed3, side: Ask, order_type: Limit, price: BigDecimal("1.03"), qty: BigDecimal("0.35"), ts: SystemTime { tv_sec: 1695144728, tv_nsec: 283489000 } }), Ok(Filled { order_id: 315c9c8a-15d8-411f-8683-24c76090af10, side: Bid, order_type: Market, price: BigDecimal("41712.60777901"), qty: BigDecimal("0.55"), ts: SystemTime { tv_sec: 1695144728, tv_nsec: 283492000 } }), Ok(PartiallyFilled { order_id: b7acaed6-4036-4cba-a81d-a975f1142e4f, side: Ask, order_type: Limit, price: BigDecimal("41712.60777901"), qty: BigDecimal("0.55"), ts: SystemTime { tv_sec: 1695144728, tv_nsec: 283492000 } })]
Spread => bid: 1.01, ask: 41712.60777901

Order => NewLimitOrder { order_id: 7a387864-7399-4f25-bfdd-e15aa152f38b, order_asset: BTC, price_asset: USD, side: Ask, price: BigDecimal("1.05"), qty: BigDecimal("0.5"), ts: SystemTime { tv_sec: 1695144728, tv_nsec: 283307000 } }
Processing => [Ok(Accepted { order_id: 7a387864-7399-4f25-bfdd-e15aa152f38b, order_asset: BTC, order_type: Limit, price_asset: USD, price: Some(BigDecimal("1.05")), qty: BigDecimal("0.5"), side: Ask, ts: SystemTime { tv_sec: 1695144728, tv_nsec: 283525000 } })]
Spread => bid: 1.01, ask: 1.05

Order => NewLimitOrder { order_id: c2991766-6fef-4456-bd86-05f4f83703fe, order_asset: BTC, price_asset: USD, side: Bid, price: BigDecimal("1.06"), qty: BigDecimal("0.6"), ts: SystemTime { tv_sec: 1695144728, tv_nsec: 283311000 } }
Processing => [Ok(Accepted { order_id: c2991766-6fef-4456-bd86-05f4f83703fe, order_asset: BTC, order_type: Limit, price_asset: USD, price: Some(BigDecimal("1.06")), qty: BigDecimal("0.6"), side: Bid, ts: SystemTime { tv_sec: 1695144728, tv_nsec: 283540000 } }), Ok(PartiallyFilled { order_id: c2991766-6fef-4456-bd86-05f4f83703fe, side: Bid, order_type: Limit, price: BigDecimal("1.05"), qty: BigDecimal("0.5"), ts: SystemTime { tv_sec: 1695144728, tv_nsec: 283541000 } }), Ok(Filled { order_id: 7a387864-7399-4f25-bfdd-e15aa152f38b, side: Ask, order_type: Limit, price: BigDecimal("1.05"), qty: BigDecimal("0.5"), ts: SystemTime { tv_sec: 1695144728, tv_nsec: 283541000 } })]
Spread => bid: 1.06, ask: 41712.60777901
```