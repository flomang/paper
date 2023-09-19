// use library::utils::{serialize_bigdecimal, serialize_bigdecimal_opt};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::time::SystemTime;
use uuid::Uuid;
use bigdecimal::{BigDecimal, ToPrimitive};
use serde::ser::Serializer;


use super::domain::{Order, OrderSide, OrderType};
use super::order_queues::OrderQueue;
use super::orders::OrderRequest;
use super::validation::OrderRequestValidator;

const MAX_STALLED_INDICES_IN_QUEUE: u64 = 10;
const ORDER_QUEUE_INIT_CAPACITY: usize = 500;

pub type OrderProcessingResult<Asset> = Vec<Result<Success<Asset>, Failed>>;

fn serialize_bigdecimal_opt<S>(bg: &Option<BigDecimal>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match bg {
     Some(b) => serializer.serialize_f64(b.to_f64().unwrap()),
     None => serializer.serialize_none(),
    }
}

fn serialize_bigdecimal<S>(bg: &BigDecimal, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_f64(bg.to_f64().unwrap())
}


#[derive(Debug, Serialize, Deserialize)]
pub enum Success<Asset> {
    Accepted {
        order_id: Uuid,
        order_asset: Asset,
        order_type: OrderType,
        price_asset: Asset,
        #[serde(serialize_with = "serialize_bigdecimal_opt")]
        price: Option<BigDecimal>,
        #[serde(serialize_with = "serialize_bigdecimal")]
        qty: BigDecimal,
        side: OrderSide,
        ts: SystemTime,
    },

    Filled {
        order_id: Uuid,
        side: OrderSide,
        order_type: OrderType,
        #[serde(serialize_with = "serialize_bigdecimal")]
        price: BigDecimal,
        #[serde(serialize_with = "serialize_bigdecimal")]
        qty: BigDecimal,
        ts: SystemTime,
    },

    PartiallyFilled {
        order_id: Uuid,
        side: OrderSide,
        order_type: OrderType,
        #[serde(serialize_with = "serialize_bigdecimal")]
        price: BigDecimal,
        #[serde(serialize_with = "serialize_bigdecimal")]
        qty: BigDecimal,
        ts: SystemTime,
    },

    Amended {
        order_id: Uuid,
        #[serde(serialize_with = "serialize_bigdecimal")]
        price: BigDecimal,
        #[serde(serialize_with = "serialize_bigdecimal")]
        qty: BigDecimal,
        ts: SystemTime,
    },

    Cancelled {
        order_id: Uuid,
        ts: SystemTime,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Failed {
    ValidationFailed(String),
    DuplicateOrderID(Uuid),
    NoMatch(Uuid),
    OrderNotFound(Uuid),
}

pub struct Orderbook<Asset>
where
    Asset: Debug + Clone + Copy + Eq,
{
    pub order_asset: Asset,
    pub price_asset: Asset,
    pub bid_queue: OrderQueue<Order<Asset>>,
    pub ask_queue: OrderQueue<Order<Asset>>,
    order_validator: OrderRequestValidator<Asset>,
}

impl<Asset> Orderbook<Asset>
where
    Asset: Debug + Clone + Copy + Eq,
{
    /// Create new orderbook for pair of assets
    ///
    /// # Examples
    ///
    /// Basic usage:
    /// ```
    //let mut orderbook = Orderbook::new(Asset::BTC, Asset::USD);
    //let result = orderbook.process_order(OrderRequest::MarketOrder{  });
    //assert_eq!(orderbook)
    /// ```
    // todo fix doc test!
    pub fn new(order_asset: Asset, price_asset: Asset) -> Self {
        Orderbook {
            order_asset,
            price_asset,
            bid_queue: OrderQueue::new(
                OrderSide::Bid,
                MAX_STALLED_INDICES_IN_QUEUE,
                ORDER_QUEUE_INIT_CAPACITY,
            ),
            ask_queue: OrderQueue::new(
                OrderSide::Ask,
                MAX_STALLED_INDICES_IN_QUEUE,
                ORDER_QUEUE_INIT_CAPACITY,
            ),
            order_validator: OrderRequestValidator::new(order_asset, price_asset),
        }
    }

    pub fn process_order(&mut self, order: OrderRequest<Asset>) -> OrderProcessingResult<Asset> {
        // processing result accumulator
        let mut proc_result: OrderProcessingResult<Asset> = vec![];

        // validate request
        if let Err(reason) = self.order_validator.validate(&order) {
            proc_result.push(Err(Failed::ValidationFailed(String::from(reason))));
            return proc_result;
        }

        match order {
            OrderRequest::NewMarketOrder {
                order_id,
                order_asset,
                price_asset,
                side,
                qty,
                ts: _ts,
            } => {
                proc_result.push(Ok(Success::Accepted {
                    order_id,
                    order_asset,
                    price_asset,
                    price: None,
                    order_type: OrderType::Market,
                    qty: qty.clone(),
                    side,
                    ts: SystemTime::now(),
                }));

                self.process_market_order(
                    &mut proc_result,
                    order_id,
                    order_asset,
                    price_asset,
                    side,
                    qty,
                );
            }

            OrderRequest::NewLimitOrder {
                order_id,
                order_asset,
                price_asset,
                side,
                price,
                qty,
                ts,
            } => {
                proc_result.push(Ok(Success::Accepted {
                    order_id,
                    order_asset,
                    price_asset,
                    price: Some(price.clone()),
                    order_type: OrderType::Limit,
                    side,
                    qty: qty.clone(),
                    ts: SystemTime::now(),
                }));

                self.process_limit_order(
                    &mut proc_result,
                    order_id,
                    order_asset,
                    price_asset,
                    side,
                    price,
                    qty,
                    ts,
                );
            }

            OrderRequest::AmendOrder {
                id,
                side,
                price,
                qty,
                ts,
            } => {
                self.process_order_amend(&mut proc_result, id, side, price, qty, ts);
            }

            OrderRequest::CancelOrder { id, side } => {
                self.process_order_cancel(&mut proc_result, id, side);
            }
        }

        // return collected processing results
        proc_result
    }

    /// Get current spread as a tuple: (bid, ask)
    pub fn current_spread(&mut self) -> Option<(BigDecimal, BigDecimal)> {
        let bid = self.bid_queue.peek()?.price.clone();
        let ask = self.ask_queue.peek()?.price.clone();
        Some((bid, ask))
    }

    /* Processing logic */

    fn process_market_order(
        &mut self,
        results: &mut OrderProcessingResult<Asset>,
        order_id: Uuid,
        order_asset: Asset,
        price_asset: Asset,
        side: OrderSide,
        qty: BigDecimal,
    ) {
        // get copy of the current limit order
        let opposite_order_result = {
            let opposite_queue = match side {
                OrderSide::Bid => &mut self.ask_queue,
                OrderSide::Ask => &mut self.bid_queue,
            };
            opposite_queue.peek().cloned()
        };

        if let Some(opposite_order) = opposite_order_result {
            let matching_complete = self.order_matching(
                results,
                &opposite_order,
                order_id,
                order_asset,
                price_asset,
                OrderType::Market,
                side,
                qty.clone(),
            );

            if !matching_complete {
                // match the rest
                self.process_market_order(
                    results,
                    order_id,
                    order_asset,
                    price_asset,
                    side,
                    qty - opposite_order.qty,
                );
            }
        } else {
            // no limit orders found
            results.push(Err(Failed::NoMatch(order_id)));
        }
    }

    fn process_limit_order(
        &mut self,
        results: &mut OrderProcessingResult<Asset>,
        order_id: Uuid,
        order_asset: Asset,
        price_asset: Asset,
        side: OrderSide,
        price: BigDecimal,
        qty: BigDecimal,
        ts: SystemTime,
    ) {
        // take a look at current opposite limit order
        let opposite_order_result = {
            let opposite_queue = match side {
                OrderSide::Bid => &mut self.ask_queue,
                OrderSide::Ask => &mut self.bid_queue,
            };
            opposite_queue.peek().cloned()
        };

        if let Some(opposite_order) = opposite_order_result {
            let could_be_matched = match side {
                // verify bid/ask price overlap
                OrderSide::Bid => price >= opposite_order.price,
                OrderSide::Ask => price <= opposite_order.price,
            };

            if could_be_matched {
                // match immediately
                let matching_complete = self.order_matching(
                    results,
                    &opposite_order,
                    order_id,
                    order_asset,
                    price_asset,
                    OrderType::Limit,
                    side,
                    qty.clone(),
                );

                if !matching_complete {
                    // process the rest of new limit order
                    self.process_limit_order(
                        results,
                        order_id,
                        order_asset,
                        price_asset,
                        side,
                        price,
                        qty - opposite_order.qty,
                        ts,
                    );
                }
            } else {
                // just insert new order in queue
                self.store_new_limit_order(
                    results,
                    order_id,
                    order_asset,
                    price_asset,
                    side,
                    price,
                    qty,
                    ts,
                );
            }
        } else {
            self.store_new_limit_order(
                results,
                order_id,
                order_asset,
                price_asset,
                side,
                price,
                qty,
                ts,
            );
        }
    }

    fn process_order_amend(
        &mut self,
        results: &mut OrderProcessingResult<Asset>,
        order_id: Uuid,
        side: OrderSide,
        price: BigDecimal,
        qty: BigDecimal,
        ts: SystemTime,
    ) {
        let order_queue = match side {
            OrderSide::Bid => &mut self.bid_queue,
            OrderSide::Ask => &mut self.ask_queue,
        };

        if order_queue.amend(
            order_id,
            price.clone(),
            ts,
            Order {
                order_id,
                order_asset: self.order_asset,
                price_asset: self.price_asset,
                side,
                price: price.clone(),
                qty: qty.clone(),
            },
        ) {
            results.push(Ok(Success::Amended {
                order_id,
                price,
                qty,
                ts: SystemTime::now(),
            }));
        } else {
            results.push(Err(Failed::OrderNotFound(order_id)));
        }
    }

    fn process_order_cancel(
        &mut self,
        results: &mut OrderProcessingResult<Asset>,
        order_id: Uuid,
        side: OrderSide,
    ) {
        let order_queue = match side {
            OrderSide::Bid => &mut self.bid_queue,
            OrderSide::Ask => &mut self.ask_queue,
        };

        if order_queue.cancel(order_id) {
            results.push(Ok(Success::Cancelled {
                order_id,
                ts: SystemTime::now(),
            }));
        } else {
            results.push(Err(Failed::OrderNotFound(order_id)));
        }
    }

    /* Helpers */

    fn store_new_limit_order(
        &mut self,
        results: &mut OrderProcessingResult<Asset>,
        order_id: Uuid,
        order_asset: Asset,
        price_asset: Asset,
        side: OrderSide,
        price: BigDecimal,
        qty: BigDecimal,
        ts: SystemTime,
    ) {
        let order_queue = match side {
            OrderSide::Bid => &mut self.bid_queue,
            OrderSide::Ask => &mut self.ask_queue,
        };
        if !order_queue.insert(
            order_id,
            price.clone(),
            ts,
            Order {
                order_id,
                order_asset,
                price_asset,
                side,
                price,
                qty,
            },
        ) {
            results.push(Err(Failed::DuplicateOrderID(order_id)))
        };
    }

    fn order_matching(
        &mut self,
        results: &mut OrderProcessingResult<Asset>,
        opposite_order: &Order<Asset>,
        order_id: Uuid,
        order_asset: Asset,
        price_asset: Asset,
        order_type: OrderType,
        side: OrderSide,
        qty: BigDecimal,
    ) -> bool {
        // real processing time
        let deal_time = SystemTime::now();

        // match immediately
        if qty < opposite_order.qty {
            // fill new limit and modify opposite limit

            // report filled new order
            results.push(Ok(Success::Filled {
                order_id,
                side,
                order_type,
                price: opposite_order.price.clone(),
                qty: qty.clone(),
                ts: deal_time,
            }));

            // report partially filled opposite limit order
            results.push(Ok(Success::PartiallyFilled {
                order_id: opposite_order.order_id,
                side: opposite_order.side,
                order_type: OrderType::Limit,
                price: opposite_order.price.clone(),
                qty: qty.clone(),
                ts: deal_time,
            }));

            // modify unmatched part of the opposite limit order
            {
                let opposite_queue = match side {
                    OrderSide::Bid => &mut self.ask_queue,
                    OrderSide::Ask => &mut self.bid_queue,
                };
                opposite_queue.modify_current_order(Order {
                    order_id: opposite_order.order_id,
                    order_asset,
                    price_asset,
                    side: opposite_order.side,
                    price: opposite_order.price.clone(),
                    qty: opposite_order.qty.clone() - qty,
                });
            }
        } else if qty > opposite_order.qty {
            // partially fill new limit order, fill opposite limit and notify to process the rest

            // report new order partially filled
            results.push(Ok(Success::PartiallyFilled {
                order_id,
                side,
                order_type,
                price: opposite_order.price.clone(),
                qty: opposite_order.qty.clone(),
                ts: deal_time,
            }));

            // report filled opposite limit order
            results.push(Ok(Success::Filled {
                order_id: opposite_order.order_id,
                side: opposite_order.side,
                order_type: OrderType::Limit,
                price: opposite_order.price.clone(),
                qty: opposite_order.qty.clone(),
                ts: deal_time,
            }));

            // remove filled limit order from the queue
            {
                let opposite_queue = match side {
                    OrderSide::Bid => &mut self.ask_queue,
                    OrderSide::Ask => &mut self.bid_queue,
                };
                opposite_queue.pop();
            }

            // matching incomplete
            return false;
        } else {
            // orders exactly match -> fill both and remove old limit

            // report filled new order
            results.push(Ok(Success::Filled {
                order_id,
                side,
                order_type,
                price: opposite_order.price.clone(),
                qty: qty.clone(),
                ts: deal_time,
            }));
            // report filled opposite limit order
            results.push(Ok(Success::Filled {
                order_id: opposite_order.order_id,
                side: opposite_order.side,
                order_type: OrderType::Limit,
                price: opposite_order.price.clone(),
                qty,
                ts: deal_time,
            }));

            // remove filled limit order from the queue
            {
                let opposite_queue = match side {
                    OrderSide::Bid => &mut self.ask_queue,
                    OrderSide::Ask => &mut self.bid_queue,
                };
                opposite_queue.pop();
            }
        }

        // complete matching
        true
    }
}

#[cfg(test)]
mod test {

    use super::super::orders;
    use bigdecimal::Zero;
    use std::str::FromStr;

    use super::*;

    #[derive(PartialEq, Eq, Debug, Copy, Clone)]
    pub enum Asset {
        USD,
        BTC,
    }

    fn bigdec(num: &str) -> BigDecimal {
        match BigDecimal::from_str(num) {
            Ok(dec) => dec,
            Err(_) => BigDecimal::zero(),
        }
    }

    #[test]
    fn cancel_nonexisting() {
        let id = Uuid::parse_str("00000000-0000-0000-0000-000000000000").unwrap();
        let mut orderbook = Orderbook::new(Asset::BTC, Asset::USD);
        let request = orders::limit_order_cancel_request(id, OrderSide::Bid);
        let mut result = orderbook.process_order(request);

        assert_eq!(result.len(), 1);
        match result.pop().unwrap() {
            Err(_) => (),
            _ => panic!("unexpected events"),
        }
    }

    #[test]
    fn amend_order() {
        let btc_asset = Asset::BTC;
        let usd_asset = Asset::USD;
        let mut orderbook = Orderbook::new(btc_asset, usd_asset);
        let limit_order = orders::new_limit_order_request(
            btc_asset,
            usd_asset,
            OrderSide::Bid,
            bigdec("41711.760112"),
            bigdec("0.15"),
            SystemTime::now(),
        );

        let mut results = orderbook.process_order(limit_order);
        assert_eq!(results.len(), 1);

        if let Success::Accepted {
            order_id,
            order_asset: _,
            price_asset: _,
            price: _,
            order_type: _,
            side: _,
            qty: _,
            ts: _,
        } = results
            .pop()
            .expect("expected a Result")
            .expect("this should be Success")
        {
            let amend_order = orders::amend_order_request(
                order_id,
                OrderSide::Bid,
                bigdec("40000.00"),
                bigdec("0.16"),
                SystemTime::now(),
            );

            let mut results2 = orderbook.process_order(amend_order);
            assert_eq!(results2.len(), 1);

            let order = orderbook.bid_queue.peek().unwrap();
            assert_eq!(order.order_id, order_id);
            assert_eq!(order.price, bigdec("40000.00"));
            assert_eq!(order.qty, bigdec("0.16"));

            if let Success::Amended {
                order_id: _,
                price,
                qty,
                ts: _,
            } = results2
                .pop()
                .expect("expected a Result")
                .expect("this should be Success")
            {
                assert_eq!(price, bigdec("40000.00"));
                assert_eq!(qty, bigdec("0.16"));
            }
        }
    }

    #[test]
    fn request_list() {
        let btc_asset = Asset::BTC;
        let usd_asset = Asset::USD;
        let mut orderbook = Orderbook::new(btc_asset, usd_asset);
        let request_list = vec![
            orders::new_limit_order_request(
                btc_asset,
                usd_asset,
                OrderSide::Bid,
                bigdec("41711.760112"),
                bigdec("0.15"),
                SystemTime::now(),
            ),
            orders::new_limit_order_request(
                btc_asset,
                usd_asset,
                OrderSide::Ask,
                bigdec("41712.60777901"),
                bigdec("1.0223"),
                SystemTime::now(),
            ),
            orders::new_limit_order_request(
                btc_asset,
                usd_asset,
                OrderSide::Bid,
                bigdec("1.01"),
                bigdec("0.4"),
                SystemTime::now(),
            ),
            orders::new_limit_order_request(
                btc_asset,
                usd_asset,
                OrderSide::Ask,
                bigdec("1.03"),
                bigdec("0.5"),
                SystemTime::now(),
            ),
            orders::new_market_order_request(
                btc_asset,
                usd_asset,
                OrderSide::Bid,
                bigdec("0.90"),
                SystemTime::now(),
            ),
            orders::new_limit_order_request(
                btc_asset,
                usd_asset,
                OrderSide::Ask,
                bigdec("1.05"),
                bigdec("0.5"),
                SystemTime::now(),
            ),
            orders::new_limit_order_request(
                btc_asset,
                usd_asset,
                OrderSide::Bid,
                bigdec("1.06"),
                bigdec("0.6"),
                SystemTime::now(),
            ),
        ];
        for order in request_list {
            let results = orderbook.process_order(order);
            for result in results {
                println!("\tResult => {:?}", result);
            }

            if let Some((bid, ask)) = orderbook.current_spread() {
                println!("Spread => bid: {}, ask: {}\n", bid, ask);
            }
        }
    }
}
