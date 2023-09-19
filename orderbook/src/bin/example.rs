
extern crate orderbook;

use std::{time::SystemTime, str::FromStr};

use bigdecimal::{BigDecimal, Zero};
use orderbook::guid::{orderbook::Orderbook, orders, domain::OrderSide};

#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub enum BrokerAsset {
    USD,
    EUR,
    BTC,
    ETH,
}


fn parse_asset(asset: &str) -> Option<BrokerAsset> {
    match asset {
        "USD" => Some(BrokerAsset::USD),
        "EUR" => Some(BrokerAsset::EUR),
        "BTC" => Some(BrokerAsset::BTC),
        "ETH" => Some(BrokerAsset::ETH),
        _ => None,
    }
}

fn big_decimal(num: &str) -> BigDecimal {
    match BigDecimal::from_str(num) {
        Ok(dec) => dec,
        Err(_) => BigDecimal::zero(),
    }
}


fn main() {
    let mut orderbook = Orderbook::new(BrokerAsset::BTC, BrokerAsset::USD);
    let btc_asset = parse_asset("BTC").unwrap();
    let usd_asset = parse_asset("USD").unwrap();

    // create order requests
    let order_list = vec![
            orders::new_limit_order_request(
                btc_asset,
                usd_asset,
                OrderSide::Bid,
                big_decimal("41711.760112"),
                big_decimal("0.15"),
                SystemTime::now(),
            ),
            orders::new_limit_order_request(
                btc_asset,
                usd_asset,
                OrderSide::Ask,
                big_decimal("41712.60777901"),
                big_decimal("1.0223"),
                SystemTime::now(),
            ),
            orders::new_limit_order_request(
                btc_asset,
                usd_asset,
                OrderSide::Bid,
                big_decimal("1.01"),
                big_decimal("0.4"),
                SystemTime::now(),
            ),
            orders::new_limit_order_request(
                btc_asset,
                usd_asset,
                OrderSide::Ask,
                big_decimal("1.03"),
                big_decimal("0.5"),
                SystemTime::now(),
            ),
            orders::new_market_order_request(
                btc_asset,
                usd_asset,
                OrderSide::Bid,
                big_decimal("0.90"),
                SystemTime::now(),
            ),
            orders::new_limit_order_request(
                btc_asset,
                usd_asset,
                OrderSide::Ask,
                big_decimal("1.05"),
                big_decimal("0.5"),
                SystemTime::now(),
            ),
            orders::new_limit_order_request(
                btc_asset,
                usd_asset,
                OrderSide::Bid,
                big_decimal("1.06"),
                big_decimal("0.6"),
                SystemTime::now(),
            ),
        ]; 

    // processing
    for order in order_list {
        println!("Order => {:?}", &order);
        let res = orderbook.process_order(order);
        println!("Processing => {:?}", res);
        if let Some((bid, ask)) = orderbook.current_spread() {
            println!("Spread => bid: {}, ask: {}\n", bid, ask);
        } else {
            println!("Spread => not available\n");
        }
    }
}