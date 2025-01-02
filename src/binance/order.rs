use crate::error::Result;
use reqwest::Method;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Serialize, Deserialize, Debug)]
pub struct OrderResponse {
    pub orderId: u64,

    origQty: String,
    price: String,
    reduceOnly: bool,
    side: String,
    positionSide: String,
    status: String,
    stopPrice: String,
    symbol: String,
    timeInForce: String,
    #[serde(rename = "type")]
    orderType: String,
    origType: String,
    updateTime: i64,
    workingType: String,
    // 添加其他需要的字段
}

pub async fn create_order(
    symbol: &str,
    side: &str,          // 买入或卖出： "BUY" 或 "SELL"
    position_side: &str, // 仓位方向，例如 "LONG" 或 "SHORT"
    order_type: &str,    // 订单类型，例如 "LIMIT" 或 "MARKET"
    quantity: &str,      // 下单数量
    price: Option<&str>, // 价格，市价单可为空
    stop_price: Option<&str>,
) -> Result<OrderResponse> {
    let endpoint = format!("{}/fapi/v1/order", super::BASE_URL);

    // 获取当前时间戳
    let timestamp = super::create_timestamp();

    // 构建查询字符串
    let mut query_string = format!(
        "symbol={}&side={}&positionSide={}&type={}&quantity={}&newOrderRespType={}&timestamp={}",
        symbol, side, position_side, order_type, quantity, "RESULT", timestamp
    );

    if let Some(p) = price {
        query_string.push_str(&format!("&price={}&timeInForce={}", p, "GTC"));
    }
    if let Some(sp) = stop_price {
        query_string.push_str(&format!("&stopPrice={}", sp));
    }

    // 生成签名
    let signature = super::create_signature(&super::API_SECRET, &query_string);
    let url = format!("{}?{}&signature={}", endpoint, query_string, signature);

    // 使用 post_request 发送带 body 的请求
    let response = super::request::<OrderResponse>(&url, Method::POST, &super::API_KEY).await?;

    Ok(response)
}

pub async fn get_orders(symbol: &str) -> Result<Vec<OrderResponse>> {
    let endpoint = format!("{}/fapi/v1/allOrders", super::BASE_URL);

    // 获取当前时间戳
    let timestamp = super::create_timestamp();

    // 准备查询字符串并生成签名
    let query_string = format!("symbol={}&timestamp={}", symbol, timestamp);
    let signature = super::create_signature(&super::API_SECRET, &query_string);

    // 完整请求 URL，包含签名
    let url = format!("{}?{}&signature={}", endpoint, query_string, signature);

    // 调用 get_request 发起请求并解析为 AccountInfo
    super::request::<Vec<OrderResponse>>(&url, Method::GET, &super::API_KEY).await
}

#[derive(Debug, Deserialize)]
pub struct CancelOrderResponse {
    // 根据 API 文档定义响应字段
    orderId: u64,
}

pub async fn cancel_order(symbol: &str, order_id: u64) -> Result<CancelOrderResponse> {
    let endpoint = format!("{}/fapi/v1/order", super::BASE_URL);

    // 获取当前时间戳
    let timestamp = super::create_timestamp();

    // 准备查询字符串并生成签名
    let query_string = format!(
        "symbol={}&orderId={}&timestamp={}",
        symbol, order_id, timestamp
    );
    let signature = super::create_signature(&super::API_SECRET, &query_string);

    // 完整请求 URL，包含签名
    let url = format!("{}?{}&signature={}", endpoint, query_string, signature);

    // 调用 get_request 发起请求并解析为 AccountInfo
    super::request::<CancelOrderResponse>(&url, Method::DELETE, &super::API_KEY).await
}
