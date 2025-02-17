use crate::error::Result;
use reqwest::Method;

use crate::models::record_model::TradeRecord;

pub async fn get_order_record_api(
    symbol: &str,
    order_id: u64,
    key: &str,
    secret: &str,
) -> Result<Vec<TradeRecord>> {
    let endpoint = format!("{}/fapi/v1/userTrades", super::BASE_URL);

    // 获取当前时间戳
    let timestamp = super::create_timestamp();

    // 准备查询字符串并生成签名
    let query_string = format!(
        "symbol={}&orderId={}&timestamp={}",
        symbol, order_id, timestamp
    );
    let signature = super::create_signature(secret, &query_string);

    // 完整请求 URL，包含签名
    let url = format!("{}?{}&signature={}", endpoint, query_string, signature);

    // 调用 get_request 发起请求并解析为 AccountInfo
    super::request(&url, Method::GET, key).await
}
