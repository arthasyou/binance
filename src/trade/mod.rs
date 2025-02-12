use std::fmt;
use std::time::Duration;

use sea_orm::{ActiveModelTrait, DatabaseConnection, Set};
use serde::{Deserialize, Serialize};
use tokio::time::sleep;

use crate::binance::account::get_order;
use crate::binance::order::{cancel_order, create_order, CancelOrderResponse, OrderResponse};
use crate::error::{Error, Result};
use crate::orm::trades;

// 模拟的交易方向
#[derive(Debug, PartialEq, Deserialize, Serialize, Clone)]
pub enum TradeDirection {
    Long,  // 做多
    Short, // 做空
}

impl fmt::Display for TradeDirection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TradeDirection::Long => write!(f, "Long"),
            TradeDirection::Short => write!(f, "Short"),
        }
    }
}

// 模拟的交易类型
#[derive(Debug, Clone, Serialize)]
pub struct Trade {
    pub id: usize,
    pub order_id: u64,
    pub stop_order: u64,           // 唯一ID字段，用于唯一标识每笔交易
    pub symbol: String, // 货币或资产符号，表示此交易涉及的交易品种，如 "EUR/USD" 或 "AAPL"
    pub entry_price: f64, // 入场价格，交易开始时的初始价格
    pub stop_loss: f64, // 止损点位，如果当前价格达到该值，交易将自动平仓以限制损失
    highest_price: f64, // 记录历史最高价格，用于动态调整止损点和判断利润情况（做多时）
    lowest_price: f64,  // 记录历史最低价格，用于动态调整止损点和判断利润情况（做空时）
    pub direction: TradeDirection, // 交易方向，标识是做多还是做空
    pub quantity: String,
    pub leverage: f64,
    pub adjustment: Vec<Adjustment>,
    pub is_closed: bool, // 杠杆倍数
}

impl Trade {
    // 创建一个新的交易，自动设置止损为-5%（即95%）
    pub async fn new(
        id: usize,
        order_id: u64,
        symbol: String,
        entry_price: f64,
        direction: TradeDirection,
        quantity: String,
        leverage: f64,
        stop_loss_percent: f64,
        mut adjustment: Vec<Adjustment>,
    ) -> Self {
        let stop_loss = calculate_stop_price(&direction, entry_price, leverage, stop_loss_percent);
        // let (side, position_side) = match direction {
        //     TradeDirection::Long => ("SELL", "LONG"),
        //     TradeDirection::Short => ("BUY", "SHORT"),
        // };

        // 调用 create_order_with_retry 方法设置止损单
        // let stop_order_response = create_order_with_retry(
        //     &symbol,                      // 交易对
        //     side,                         // 方向 (卖出止损或买入止损)
        //     position_side,                // 仓位方向 (LONG/SHORT)
        //     &quantity,                    // 下单数量
        //     format_stop_price(stop_loss), // 止损价格
        // )
        // .await
        // .expect("Failed to place stop order");

        let stop_order_id = order_id;

        adjustment.push(Adjustment {
            min: 1.1,
            max: None,
            adjustment: 0.1,
        });

        Self {
            id,
            order_id,
            stop_order: stop_order_id,
            symbol,
            entry_price,
            stop_loss,
            highest_price: entry_price, // 做多时初始为入场价
            lowest_price: entry_price,  // 做空时初始为入场价
            direction,
            quantity,
            leverage,
            adjustment,
            is_closed: false,
        }
    }

    // 更新价格并调整历史最高或最低价和止损
    pub async fn update_price(
        &mut self,
        book_price: (String, String),
        database: &DatabaseConnection,
    ) {
        let price = match self.direction {
            TradeDirection::Long => {
                let price = book_price.1;
                let price_f64: f64 = price.parse().unwrap();
                if price_f64 > self.highest_price {
                    self.highest_price = price_f64;
                    let profit_percentage =
                        (self.highest_price - self.entry_price) / self.entry_price;
                    self.update_stop_loss(profit_percentage, true).await;
                }
                price
            }
            TradeDirection::Short => {
                let price = book_price.0;
                let price_f64: f64 = price.parse().unwrap();
                if price_f64 < self.lowest_price {
                    self.lowest_price = price_f64;
                    let profit_percentage =
                        (self.entry_price - self.lowest_price) / self.entry_price;
                    self.update_stop_loss(profit_percentage, false).await;
                }
                price
            }
        };
        self.check_exit_conditions(&price, database).await;
    }

    async fn update_stop_loss(&mut self, profit_percentage: f64, is_long: bool) {
        let new_stop_price = self.calculate_new_stop_loss(profit_percentage, is_long);

        if new_stop_price != self.stop_loss {
            self.stop_loss = new_stop_price;
            // let _ = cancel_order_with_retry(&self.symbol, self.stop_order).await;
            // let (side, position_side) = if is_long {
            //     ("SELL", "LONG")
            // } else {
            //     ("BUY", "SHORT")
            // };
            // if let Ok(stop_order_response) = create_order_with_retry(
            //     &self.symbol,                      // 交易对
            //     side,                              // 方向 (卖出止损或买入止损)
            //     position_side,                     // 仓位方向 (LONG/SHORT)
            //     &self.quantity,                    // 下单数量
            //     format_stop_price(self.stop_loss), // 止损价格
            // )
            // .await
            // {
            //     self.stop_order = stop_order_response.orderId;
            // }
        }
    }

    fn calculate_new_stop_loss(&mut self, profit_percentage: f64, is_long: bool) -> f64 {
        let actual_price_change_percentage = profit_percentage * self.leverage;

        let adjustment = get_adjustment(actual_price_change_percentage, &mut self.adjustment);

        if adjustment == 0.0 {
            return self.stop_loss;
        }
        // let adjustment = match actual_price_change_percentage {
        //     x if x >= 0.10 && x < 0.19 => 0.02,
        //     x if x >= 0.20 && x < 0.29 => 0.04,
        //     x if x >= 0.30 && x < 0.39 => 0.09,
        //     x if x >= 0.40 && x < 0.49 => 0.16,
        //     x if x >= 0.50 && x < 0.59 => 0.25,
        //     x if x >= 0.60 && x < 0.69 => 0.36,
        //     x if x >= 0.70 && x < 0.79 => 0.49,
        //     x if x >= 0.7999 && x < 0.89 => 0.64,
        //     x if x >= 0.8999 && x < 1.0 => 0.81,
        //     x if x >= 0.9999 && x < 1.1 => 0.90,
        //     x if x >= 1.1 => 0.1,
        //     _ => return self.stop_loss,
        // };

        if is_long {
            if actual_price_change_percentage >= 1.09 {
                self.highest_price * (1.0 - adjustment / self.leverage)
            } else {
                self.entry_price * (1.0 + adjustment / self.leverage)
            }
        } else {
            if actual_price_change_percentage >= 1.09 {
                self.lowest_price * (1.0 + adjustment / self.leverage)
            } else {
                self.entry_price * (1.0 - adjustment / self.leverage)
            }
        }
    }

    // 检查是否应平仓
    async fn check_exit_conditions(&mut self, price: &str, database: &DatabaseConnection) {
        // 如果交易已平仓，直接返回，不打印
        if self.is_closed {
            return;
        }
        let price_f64: f64 = price.parse().unwrap();

        if (self.direction == TradeDirection::Long && price_f64 <= self.stop_loss)
            || (self.direction == TradeDirection::Short && price_f64 >= self.stop_loss)
        {
            println!(
                "止损触发于 {}，交易对 {}， 方向{:?}, 开仓价格: {}, 关闭交易 ID {}。",
                price, self.symbol, self.direction, self.entry_price, self.id
            );
            let (side, position_side) = match self.direction {
                TradeDirection::Long => ("SELL", "LONG"),
                TradeDirection::Short => ("BUY", "SHORT"),
            };
            match create_order(
                &self.symbol,
                side,
                position_side,
                "MARKET",       // 假设使用市价单
                &self.quantity, // 将数量格式化为字符串
                None,           // 市价单无需价格
                None,           // 此示例未设置止损价格
            )
            .await
            {
                Ok(order) => match get_order(&self.symbol, order.orderId).await {
                    Ok(b_order) => create_trade_record(database, &self, &b_order.avgPrice).await,
                    Err(_) => create_trade_record(database, &self, price).await,
                },
                Err(_) => {}
            }

            // 设置为已平仓状态
            self.is_closed = true;
        }
    }
}

pub async fn create_trade_record(database: &DatabaseConnection, trade: &Trade, price: &str) {
    let new_pool = trades::ActiveModel {
        symbol: Set(trade.symbol.clone()),
        entry_price: Set(trade.entry_price.to_string()),
        close_price: Set(price.to_string()),
        direction: Set(trade.direction.to_string()),
        quantity: Set(trade.quantity.clone()),
        leverage: Set(trade.leverage.to_string()),
        ..Default::default()
    };
    let _ = new_pool.insert(database).await.unwrap();
}

pub fn calculate_stop_price(
    direction: &TradeDirection,
    price: f64,
    leverage: f64,
    stop_loss_percent: f64,
) -> f64 {
    match direction {
        TradeDirection::Long => price * (1.0 - stop_loss_percent / leverage), // 做多时根据杠杆倍数和调整参数设置止损
        TradeDirection::Short => price * (1.0 + stop_loss_percent / leverage), // 做空时根据杠杆倍数和调整参数设置止损
    }
}

// pub async fn create_order_with_retry(
//     symbol: &str,
//     side: &str,
//     position_side: &str,
//     quantity: &str,
//     price: &str,
// ) -> Result<OrderResponse> {
//     // 调用 create_order 函数
//     let order_response = create_order(
//         symbol,
//         side,
//         position_side,
//         "LIMIT",     // 假设使用止损市价单
//         quantity,    // 下单数量
//         Some(price), // 市价单无需价格
//         None,        // 设置止损价格
//     )
//     .await;

//     match order_response {
//         Ok(response) => {
//             return Ok(response);
//         }
//         Err(e) => {
//             // 如果失败，打印错误并等待
//             println!("Failed to place order: {}. Retrying...", e);
//             Err(Error::SystemError("faild".to_string()))
//         }
//     }
// }

// pub async fn cancel_order_with_retry(symbol: &str, order: u64) -> Result<CancelOrderResponse> {
//     loop {
//         // 调用 create_order 函数
//         let order_response = cancel_order(symbol, order).await;

//         match order_response {
//             Ok(response) => {
//                 return Ok(response);
//             }
//             Err(e) => {
//                 // 如果失败，打印错误并等待
//                 println!("Failed to place order: {}. Retrying...", e);
//                 sleep(Duration::from_secs(1)).await; // 等待 2 秒后重试
//             }
//         }
//     }
// }

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Adjustment {
    pub min: f64,
    pub max: Option<f64>,
    pub adjustment: f64,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct AdjustmentConfig {
    pub adjustments: Vec<Adjustment>,
}

fn get_adjustment(percentage: f64, adjustments: &mut Vec<Adjustment>) -> f64 {
    adjustments.retain(|adj| percentage <= adj.max.unwrap_or(f64::INFINITY));

    adjustments
        .iter()
        .find(|adj| percentage >= adj.min && adj.max.map_or(true, |max| percentage < max))
        .map_or_else(|| 0.0, |adj| adj.adjustment)
}

#[cfg(test)]
mod tests {
    use super::*;
    // use std::f64::EPSILON;
    const EPSILON: f64 = 1e-5;

    #[test]
    fn test_calculate_new_stop_loss_long() {
        let adjustment = vec![
            Adjustment {
                min: 0.10,
                max: Some(0.19),
                adjustment: 0.02,
            },
            Adjustment {
                min: 0.20,
                max: Some(0.29),
                adjustment: 0.04,
            },
            Adjustment {
                min: 0.30,
                max: Some(0.39),
                adjustment: 0.09,
            },
            Adjustment {
                min: 0.40,
                max: Some(0.49),
                adjustment: 0.16,
            },
            Adjustment {
                min: 0.50,
                max: Some(0.59),
                adjustment: 0.25,
            },
            Adjustment {
                min: 0.60,
                max: Some(0.69),
                adjustment: 0.36,
            },
            Adjustment {
                min: 0.70,
                max: Some(0.79),
                adjustment: 0.49,
            },
            Adjustment {
                min: 0.7999,
                max: Some(0.89),
                adjustment: 0.64,
            },
            Adjustment {
                min: 0.8999,
                max: Some(1.0),
                adjustment: 0.81,
            },
            Adjustment {
                min: 0.9999,
                max: Some(1.1),
                adjustment: 0.90,
            },
            Adjustment {
                min: 1.1,
                max: None,
                adjustment: 0.1,
            },
        ];
        let mut trade = Trade {
            entry_price: 4.5,
            highest_price: 5.0,
            lowest_price: 4.0,
            leverage: 10.0,
            stop_loss: 4.0,
            id: 1,
            order_id: 1,
            stop_order: 1,
            symbol: "Filusdt".to_string(),
            direction: TradeDirection::Long,
            quantity: "1.0".to_string(),
            adjustment,
            is_closed: false,
        };

        let test_cases = vec![
            (0.009, 4.0, "No change for profit < 10%"),
            (0.010, 4.509, "Profit 10%"),
            (0.020, 4.518, "Profit 20%"),
            (0.030, 4.5405, "Profit 30%"),
            (0.040, 4.572, "Profit 40%"),
            (0.050, 4.6125, "Profit 50%"),
            (0.060, 4.662, "Profit 60%"),
            (0.070, 4.7205, "Profit 70%"),
            (0.080, 4.788, "Profit 80%"),
            (0.090, 4.8645, "Profit 90%"),
            (0.1, 4.905, "Profit 100%"),
            (0.12, 4.95, "Profit 120%"),
        ];

        for (profit, expected, description) in test_cases {
            let result = trade.calculate_new_stop_loss(profit, true);
            // let EPSILON = 1e-9;
            assert!(
                (result - expected).abs() <= EPSILON,
                "{}: Expected {:.4}, got {:.4}",
                description,
                expected,
                result
            );
        }
    }

    #[test]
    fn test_calculate_new_stop_loss_short() {
        let adjustment = vec![
            Adjustment {
                min: 0.10,
                max: Some(0.19),
                adjustment: 0.02,
            },
            Adjustment {
                min: 0.20,
                max: Some(0.29),
                adjustment: 0.04,
            },
            Adjustment {
                min: 0.30,
                max: Some(0.39),
                adjustment: 0.09,
            },
            Adjustment {
                min: 0.40,
                max: Some(0.49),
                adjustment: 0.16,
            },
            Adjustment {
                min: 0.50,
                max: Some(0.59),
                adjustment: 0.25,
            },
            Adjustment {
                min: 0.60,
                max: Some(0.69),
                adjustment: 0.36,
            },
            Adjustment {
                min: 0.70,
                max: Some(0.79),
                adjustment: 0.49,
            },
            Adjustment {
                min: 0.7999,
                max: Some(0.89),
                adjustment: 0.64,
            },
            Adjustment {
                min: 0.8999,
                max: Some(1.0),
                adjustment: 0.81,
            },
            Adjustment {
                min: 0.9999,
                max: Some(1.1),
                adjustment: 0.90,
            },
            Adjustment {
                min: 1.1,
                max: None,
                adjustment: 0.1,
            },
        ];
        let mut trade = Trade {
            entry_price: 4.5,
            highest_price: 5.0,
            lowest_price: 4.0,
            leverage: 10.0,
            stop_loss: 5.0,
            id: 1,
            order_id: 1,
            stop_order: 1,
            symbol: "Filusdt".to_string(),
            direction: TradeDirection::Short,
            quantity: "1.0".to_string(),
            adjustment,
            is_closed: false,
        };

        // x if x >= 0.10 && x < 0.19 => 0.02,
        // x if x >= 0.20 && x < 0.29 => 0.04,
        // x if x >= 0.30 && x < 0.39 => 0.09,
        // x if x >= 0.40 && x < 0.49 => 0.16,
        // x if x >= 0.50 && x < 0.59 => 0.25,
        // x if x >= 0.60 && x < 0.69 => 0.36,
        // x if x >= 0.70 && x < 0.79 => 0.49,
        // x if x >= 0.7999 && x < 0.89 => 0.64,
        // x if x >= 0.8999 && x < 1.0 => 0.81,
        // x if x >= 0.9999 && x < 1.1 => 0.90,
        // x if x >= 1.1 => 0.1,
        // _ => return self.stop_loss,

        let test_cases = vec![
            (0.009, 5.0, "No change for profit < 10%"),
            (0.010, 4.491, "Profit 10%"),
            (0.020, 4.482, "Profit 20%"),
            (0.030, 4.4595, "Profit 30%"),
            (0.040, 4.428, "Profit 40%"),
            (0.050, 4.3875, "Profit 50%"),
            (0.060, 4.338, "Profit 60%"),
            (0.070, 4.2795, "Profit 70%"),
            (0.080, 4.212, "Profit 80%"),
            (0.090, 4.1355, "Profit 90%"),
            (0.10, 4.095, "Profit 100%"),
            (0.12, 4.04, "Profit 120%"),
        ];

        for (profit, expected, description) in test_cases {
            let result = trade.calculate_new_stop_loss(profit, false);
            assert!(
                (result - expected).abs() < EPSILON,
                "{}: Expected {:.4}, got {:.4}",
                description,
                expected,
                result
            );
        }
    }
}
