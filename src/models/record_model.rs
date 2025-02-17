use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct GetOrderRequest {
    pub symbol: String,
    pub order_id: u64,
}

#[derive(Deserialize, Serialize)]
pub struct TradeRecord {
    buyer: bool,        // 是否是买方
    commission: String, // 手续费
    #[serde(rename = "commissionAsset")]
    commission_asset: String, // 手续费计价单位
    id: u64,            // 交易ID
    maker: bool,        // 是否是挂单方
    #[serde(rename = "orderId")]
    order_id: u64, // 订单编号
    price: String,      // 成交价
    qty: String,        // 成交量
    #[serde(rename = "quoteQty")]
    quote_qty: String, // 成交额
    #[serde(rename = "realizedPnl")]
    realized_pnl: String, // 实现盈亏
    side: String,       // 买卖方向
    #[serde(rename = "positionSide")]
    position_side: String, // 持仓方向
    symbol: String,     // 交易对
    time: u64,          // 时间
}
