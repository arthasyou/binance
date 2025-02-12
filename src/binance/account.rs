use crate::error::Result;
use reqwest::Method;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize}; // 需要引入 rust-decimal crate

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

#[derive(Debug, Serialize, Deserialize)]
pub struct Position {
    pub symbol: String, // 交易对

    #[serde(rename = "positionSide")]
    pub position_side: String, // 持仓方向

    #[serde(rename = "positionAmt")]
    pub position_amt: Decimal, // 头寸数量，正数为多，负数为空

    #[serde(rename = "entryPrice")]
    pub entry_price: Decimal, // 开仓均价

    #[serde(rename = "breakEvenPrice")]
    pub break_even_price: Decimal, // 盈亏平衡价

    #[serde(rename = "markPrice")]
    pub mark_price: Decimal, // 当前标记价格

    #[serde(rename = "unRealizedProfit")]
    pub unrealized_profit: Decimal, // 持仓未实现盈亏

    #[serde(rename = "liquidationPrice")]
    pub liquidation_price: Decimal, // 强平价格

    #[serde(rename = "isolatedMargin")]
    pub isolated_margin: Decimal, // 逐仓保证金

    pub notional: Decimal, // 头寸名义价值

    #[serde(rename = "marginAsset")]
    pub margin_asset: String, // 保证金资产类型

    #[serde(rename = "isolatedWallet")]
    pub isolated_wallet: Decimal, // 逐仓钱包余额

    #[serde(rename = "initialMargin")]
    pub initial_margin: Decimal, // 初始保证金

    #[serde(rename = "maintMargin")]
    pub maint_margin: Decimal, // 维持保证金

    #[serde(rename = "positionInitialMargin")]
    pub position_initial_margin: Decimal, // 持仓初始保证金

    #[serde(rename = "openOrderInitialMargin")]
    pub open_order_initial_margin: Decimal, // 开单初始保证金

    pub adl: i32, // ADL

    #[serde(rename = "bidNotional")]
    pub bid_notional: Decimal, // 买单名义价值

    #[serde(rename = "askNotional")]
    pub ask_notional: Decimal, // 卖单名义价值

    #[serde(rename = "updateTime")]
    pub update_time: i64, // 更新时间
}

pub async fn get_risk() -> Result<Vec<Position>> {
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

pub async fn get_order(symbol: &str, order_id: u64) -> Result<BiannceOrder> {
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
