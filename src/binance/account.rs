use crate::error::Result;
use reqwest::Method;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct AccountInfo {
    makerCommission: i32,
    takerCommission: i32,
    buyerCommission: i32,
    sellerCommission: i32,
    canTrade: bool,
    canWithdraw: bool,
    canDeposit: bool,
    // 添加其他需要的字段
}

pub async fn get_account() -> Result<AccountInfo> {
    let endpoint = format!("{}/fapi/v3/balance", super::BASE_URL);

    // 获取当前时间戳
    let timestamp = super::create_timestamp();

    // 准备查询字符串并生成签名
    let query_string = format!("timestamp={}", timestamp);
    let signature = super::create_signature(&super::API_SECRET, &query_string);

    // 完整请求 URL，包含签名
    let url = format!("{}?{}&signature={}", endpoint, query_string, signature);

    // 调用 get_request 发起请求并解析为 AccountInfo
    super::request(&url, Method::GET, &super::API_KEY).await
}

pub async fn get_risk() -> Result<AccountInfo> {
    let endpoint = format!("{}/fapi/v3/positionRisk", super::BASE_URL);

    // 获取当前时间戳
    let timestamp = super::create_timestamp();

    // 准备查询字符串并生成签名
    let query_string = format!("timestamp={}", timestamp);
    let signature = super::create_signature(&super::API_SECRET, &query_string);

    // 完整请求 URL，包含签名
    let url = format!("{}?{}&signature={}", endpoint, query_string, signature);

    // 调用 get_request 发起请求并解析为 AccountInfo
    super::request(&url, Method::GET, &super::API_KEY).await
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BiannceOrder {
    pub avgPrice: String,
    pub executedQty: String,
}

pub async fn get_orders(symbol: &str, order_id: u64) -> Result<BiannceOrder> {
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
    super::request(&url, Method::GET, &super::API_KEY).await
}
