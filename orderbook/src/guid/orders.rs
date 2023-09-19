
use std::time::SystemTime;
use std::fmt::Debug;
use bigdecimal::BigDecimal;
use uuid::Uuid;

use super::domain::OrderSide;


#[derive(Debug)]
pub enum OrderRequest<Asset>
where
    Asset: Debug + Clone,
{
    NewMarketOrder {
        order_id: Uuid,
        order_asset: Asset,
        price_asset: Asset,
        side: OrderSide,
        qty: BigDecimal,
        ts: SystemTime,
    },

    NewLimitOrder {
        order_id: Uuid,
        order_asset: Asset,
        price_asset: Asset,
        side: OrderSide,
        price: BigDecimal,
        qty: BigDecimal,
        ts: SystemTime,
    },

    AmendOrder {
        id: Uuid,
        side: OrderSide,
        price: BigDecimal,
        qty: BigDecimal,
        ts: SystemTime,
    },

    CancelOrder {
        id: Uuid,
        side: OrderSide,
        //ts: SystemTime,
    },
}


/* Constructors */


/// Create request for the new market order
pub fn new_market_order_request<Asset>(
    order_asset: Asset,
    price_asset: Asset,
    side: OrderSide,
    qty: BigDecimal,
    ts: SystemTime,
) -> OrderRequest<Asset>
where
    Asset: Debug + Clone,
{
    let order_id = Uuid::new_v4();
    OrderRequest::NewMarketOrder {
        order_id,
        order_asset,
        price_asset,
        qty,
        side,
        ts,
    }
}


/// Create request for the new limit order
pub fn new_limit_order_request<Asset>(
    order_asset: Asset,
    price_asset: Asset,
    side: OrderSide,
    price: BigDecimal,
    qty: BigDecimal,
    ts: SystemTime,
) -> OrderRequest<Asset>
where
    Asset: Debug + Clone,
{
    let order_id = Uuid::new_v4();
    OrderRequest::NewLimitOrder {
        order_id,
        order_asset,
        price_asset,
        side,
        price,
        qty,
        ts,
    }
}


/// Create request for changing price/qty for the active limit order.
///
/// Note: do not change order side!
/// Instead cancel existing order and create a new one.
pub fn amend_order_request<Asset>(
    id: Uuid,
    side: OrderSide,
    price: BigDecimal,
    qty: BigDecimal,
    ts: SystemTime,
) -> OrderRequest<Asset>
where
    Asset: Debug + Clone,
{

    OrderRequest::AmendOrder {
        id,
        side,
        price,
        qty,
        ts,
    }
}


/// Create request for cancelling active limit order
pub fn limit_order_cancel_request<Asset>(order_id: Uuid, side: OrderSide) -> OrderRequest<Asset>
where
    Asset: Debug + Clone,
{
    OrderRequest::CancelOrder { id: order_id, side }
}
