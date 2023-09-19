
use std::fmt::Debug;
use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub enum OrderSide {
    Bid,
    Ask,
}

impl OrderSide {
    pub fn from_string(side: &str) -> Option<OrderSide> {
        match side {
            "bid" => Some(OrderSide::Bid),
            "ask" => Some(OrderSide::Ask),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Order<Asset>
where
    Asset: Debug + Clone,
{
    pub order_id: u64,
    pub order_asset: Asset,
    pub price_asset: Asset,
    pub side: OrderSide,
    pub price: f64,
    pub qty: f64,
}


#[derive(Eq, PartialEq, Debug, Copy, Clone, Serialize, Deserialize)]
pub enum OrderType {
    Market,
    Limit,
}
