use crate::{
    error::{Error, Result},
    trade::{Adjustment, AdjustmentConfig},
};
use serde::{de, Deserialize, Deserializer};
use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Deserialize, Debug, Clone)]
pub struct Book {
    pub a: String,
    pub b: String,
}

pub fn parse_trade_json(json_text: &str) -> Result<Book> {
    serde_json::from_str(json_text).map_err(Error::JsonError) // 使用 map_err 将 serde_json::Error 转换为 Error::JsonError
}

// 自定义转换函数，将字符串转换为 f64
// fn string_to_f64<'de, D>(deserializer: D) -> core::result::Result<f64, D::Error>
// where
//     D: Deserializer<'de>,
// {
//     let s = String::deserialize(deserializer)?;
//     s.parse::<f64>().map_err(de::Error::custom)
// }

pub fn format_url(symbol: &str) -> String {
    // format!("wss://stream.binance.com:443/ws/{}@miniTicker", symbol)
    // format!("wss://stream.binance.com:443/ws/{}@trade", symbol)
    format!("wss://stream.binance.com:443/ws/{}@bookTicker", symbol)
}

pub struct TradeIdGenerator {
    counter: AtomicUsize,
}

impl TradeIdGenerator {
    // 创建新的生成器，从 1 开始计数
    pub fn new() -> Self {
        TradeIdGenerator {
            counter: AtomicUsize::new(1),
        }
    }

    // 获取下一个唯一的 id
    pub fn next_id(&self) -> usize {
        self.counter.fetch_add(1, Ordering::SeqCst)
    }
}

pub fn trim_trailing_zeros(input: &str) -> String {
    if input.contains('.') {
        // 如果包含小数点，去掉末尾的零
        input
            .trim_end_matches('0')
            .trim_end_matches('.')
            .to_string()
    } else {
        // 如果是整数，直接返回
        input.to_string()
    }
}

pub fn create_adjustment_config(mut adjustments: Vec<Adjustment>) -> AdjustmentConfig {
    // 添加最后一个默认 Adjustment
    adjustments.push(Adjustment {
        min: 1.1,
        max: None,
        adjustment: 0.1,
    });

    AdjustmentConfig { adjustments }
}

pub fn create_adjustment_config_raw(mut adjustments: Vec<Adjustment>) -> AdjustmentConfig {
    AdjustmentConfig { adjustments }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trim_trailing_zeros() {
        assert_eq!(trim_trailing_zeros("1.23230000"), "1.2323");
        assert_eq!(trim_trailing_zeros("1.3000"), "1.3");
        assert_eq!(trim_trailing_zeros("123.000"), "123");
        assert_eq!(trim_trailing_zeros("1.0"), "1");
        assert_eq!(trim_trailing_zeros("0.000"), "0");
        assert_eq!(trim_trailing_zeros("100"), "100");
        assert_eq!(trim_trailing_zeros("0"), "0");
        assert_eq!(trim_trailing_zeros("0.0"), "0");
        assert_eq!(trim_trailing_zeros(".0000"), ""); // Edge case: only zeros
    }
}
