use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::trade::TradeDirection;

#[derive(Debug, Deserialize, Validate)]
pub struct CreateTradeRequest {
    pub symbol: String,
    pub direction: TradeDirection,
    pub leverage: f64,
    pub margin: f64,
    pub stop_loss_percent: f64,
    pub adjustment_id: u8,
}

#[derive(Debug, Serialize, Validate)]
pub struct CreateTradeResponse {
    pub id: usize,
    pub symbol: String,
    pub direction: TradeDirection,
    pub leverage: f64,
    pub margin: f64,
    pub quantity: String,
    pub entry_price: String,
    pub stop_price: String,
}

// 平仓请求结构体
#[derive(Deserialize)]
pub struct CloseTradeRequest {
    pub id: usize,
    pub symbol: String,
}

// 平仓响应结构体
#[derive(Serialize)]
pub struct CloseTradeResponse {
    pub id: usize,
    pub symbol: String,
    pub direction: TradeDirection,
    pub entry_price: f64,
    pub close_price: String,
    pub quantity: String,
}

#[derive(Deserialize)]
pub struct TradeQueryParams {
    pub symbol: Option<String>,  // 货币符号 (可选)
    pub start_time: Option<u32>, // 起始时间戳 (可选)
    pub end_time: Option<u32>,   // 结束时间戳 (可选)
}
