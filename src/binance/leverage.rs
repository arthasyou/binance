use std::collections::HashMap;

use crate::error::{Error, Result};
use reqwest::Method;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct LeverageResponse {
    leverage: u32,
    maxNotionalValue: String,
    symbol: String,
}

pub async fn change_leverage(
    symbol: &str,  // 交易对符号，例如 "BTCUSDT"
    leverage: u32, // 杠杆倍数，范围 1 到 125
) -> Result<LeverageResponse> {
    let endpoint = format!("{}/fapi/v1/leverage", super::BASE_URL);

    // 获取当前时间戳
    let timestamp = super::create_timestamp();

    // 构建请求参数
    let query_string = format!(
        "symbol={}&leverage={}&timestamp={}",
        symbol, leverage, timestamp
    );

    // 生成签名
    let signature = super::create_signature(&super::API_SECRET, &query_string);
    let url = format!("{}?{}&signature={}", endpoint, query_string, signature);

    // 使用 post_request 发送请求
    let response = super::request::<LeverageResponse>(&url, Method::POST, &super::API_KEY).await?;

    Ok(response)
}

#[derive(Deserialize)]
pub struct ExchangeInfo {
    pub symbols: Vec<SymbolInfo>,
}

#[derive(Deserialize, Debug)]
pub struct SymbolInfo {
    pub symbol: String,
    pub quantityPrecision: u8,
}

pub async fn get_quantity_precision(symbols: &Vec<String>) -> Result<HashMap<String, u8>> {
    let endpoint = format!("{}/fapi/v1/exchangeInfo", super::BASE_URL);

    // 发起GET请求
    let response = super::request::<ExchangeInfo>(&endpoint, Method::GET, &super::API_KEY).await?;

    // 构建结果 HashMap
    let mut precision_map = HashMap::new();

    for symbol in symbols {
        let symbol_uppercase = symbol.to_uppercase();
        if let Some(symbol_info) = response
            .symbols
            .iter()
            .find(|s| s.symbol == *symbol_uppercase)
        {
            precision_map.insert(symbol.to_string(), symbol_info.quantityPrecision);
        } else {
            return Err(Error::SystemError(format!("未找到交易对：{}", symbol)));
        }
    }

    Ok(precision_map)
}
