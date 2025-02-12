use std::{collections::HashMap, sync::Arc};

use axum::{extract::Query, http::StatusCode, response::IntoResponse, Extension, Json};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};

use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

// use validator::Validate;

use crate::{
    binance::{
        account::{get_order, get_risk},
        leverage::change_leverage,
        order::create_order,
    },
    models::trade_model::{
        CloseTradeRequest, CloseTradeResponse, CreateTradeRequest, CreateTradeResponse,
        TradeQueryParams,
    },
    orm::trades,
    trade::{create_trade_record, Adjustment, AdjustmentConfig, Trade, TradeDirection},
    utils::TradeIdGenerator,
};

use crate::routes::error::AppError;

// 导入我们创建的 TradeIdGenerator

pub async fn create_trade(
    Extension(trades): Extension<Arc<HashMap<String, Mutex<Vec<Trade>>>>>,
    Extension(prices): Extension<Arc<HashMap<String, Mutex<(String, String)>>>>,
    Extension(precisions): Extension<Arc<HashMap<String, u8>>>,
    Extension(id_generator): Extension<Arc<TradeIdGenerator>>,
    Extension(adjustments): Extension<Arc<HashMap<u8, Mutex<AdjustmentConfig>>>>,
    Json(payload): Json<CreateTradeRequest>,
) -> impl IntoResponse {
    if let Some(mutex) = prices.get(&payload.symbol) {
        let book = mutex.lock().await;

        if let Some(mutex_config) = adjustments.get(&payload.adjustment_id) {
            let config = mutex_config.lock().await;
            let adjustment = config.adjustments.clone();
            let _ = change_leverage(&payload.symbol, payload.leverage as u32);

            // 获取精度，如果不存在则返回错误
            let precision = match precisions.get(&payload.symbol) {
                Some(&p) => p, // 解引用获取精度值
                None => {
                    return AppError::new(StatusCode::BAD_REQUEST, "Symbol precision not found")
                        .into_response();
                }
            };

            let (side, position_side, price) = match payload.direction {
                TradeDirection::Long => ("BUY", "LONG", &book.0),
                TradeDirection::Short => ("SELL", "SHORT", &book.1),
            };
            let price_f64: f64 = price.parse().unwrap();
            let quantity = calculate_quantity(&payload, price_f64, precision);
            // 确定方向

            // 调用 create_order 函数
            let order_response = create_order(
                &payload.symbol,
                side,
                position_side,
                "MARKET",  // 假设使用市价单
                &quantity, // 将数量格式化为字符串
                None,      // 市价单无需价格
                None,      // 此示例未设置止损价格
            )
            .await;

            match order_response {
                Ok(order) => {
                    match get_order(&payload.symbol, order.orderId).await {
                        Ok(b_order) => {
                            let price_f64: f64 = b_order.avgPrice.parse().unwrap();
                            // 获取订单 ID
                            let id = id_generator.next_id(); // 使用 id_generator 获取自增的 id
                            let t = Trade::new(
                                id,
                                order.orderId,
                                payload.symbol.clone(),
                                price_f64,
                                payload.direction.clone(),
                                quantity.clone(),
                                payload.leverage,
                                payload.stop_loss_percent,
                                adjustment,
                            )
                            .await;

                            // 保存交易
                            if let Some(mutex_vec) = trades.get(&payload.symbol) {
                                let mut vec = mutex_vec.lock().await;
                                vec.push(t.clone());

                                let result = CreateTradeResponse {
                                    id,
                                    symbol: payload.symbol,
                                    direction: payload.direction,
                                    leverage: payload.leverage,
                                    margin: payload.margin,
                                    quantity,
                                    entry_price: b_order.avgPrice.clone(),
                                    stop_price: format!("{:.4}", t.stop_loss),
                                };
                                (StatusCode::OK, Json(result)).into_response()
                            } else {
                                AppError::new(StatusCode::BAD_REQUEST, "Failed to save trade")
                                    .into_response()
                            }
                        }
                        Err(e) => {
                            AppError::new(StatusCode::BAD_REQUEST, format!("Order failed: {}", e))
                                .into_response()
                        }
                    }
                }
                Err(e) => {
                    // 处理下单错误
                    AppError::new(StatusCode::BAD_REQUEST, format!("Order failed: {}", e))
                        .into_response()
                }
            }
        } else {
            AppError::new(StatusCode::BAD_REQUEST, "failed, symbol").into_response()
        }
    } else {
        AppError::new(StatusCode::BAD_REQUEST, "failed, symbol").into_response()
    }
}

pub fn calculate_quantity(
    trade_request: &CreateTradeRequest,
    market_price: f64,
    precision: u8,
) -> String {
    // 确保市场价格有效，避免除以 0
    if market_price <= 0.0 {
        return "0.0".to_string();
    }

    // 计算可买数量
    let quantity = trade_request.margin * trade_request.leverage / market_price;

    // 动态格式化数量，保留指定的精度
    format!("{:.precision$}", quantity, precision = precision as usize)
}

pub async fn get_trade(
    Extension(trades): Extension<Arc<HashMap<String, Mutex<Vec<Trade>>>>>,
) -> impl IntoResponse {
    let mut all_trades = Vec::new();

    for (_, mutex_vec) in trades.iter() {
        let trades = mutex_vec.lock().await.clone();
        all_trades.extend(trades);
    }

    (StatusCode::OK, Json(all_trades)).into_response()
}

pub async fn get_price(
    Extension(prices): Extension<Arc<HashMap<String, Mutex<(String, String)>>>>,
) -> impl IntoResponse {
    // 创建一个新的 HashMap 来存储结果
    let mut all_prices = HashMap::new();

    // 遍历 `prices` 并解锁每个价格，将它们插入到 `all_prices` 中
    for (key, mutex_f64) in prices.iter() {
        let price = mutex_f64.lock().await;
        all_prices.insert(key.clone(), price.clone());
    }

    // 将结果包装成 JSON 并返回
    Json(all_prices)
}

pub async fn close_trade(
    Extension(trades): Extension<Arc<HashMap<String, Mutex<Vec<Trade>>>>>,
    Extension(prices): Extension<Arc<HashMap<String, Mutex<(String, String)>>>>,
    Extension(database): Extension<DatabaseConnection>,
    Json(payload): Json<CloseTradeRequest>,
) -> impl IntoResponse {
    // 检查是否存在该 symbol 的交易记录
    if let Some(mutex) = prices.get(&payload.symbol) {
        let book = mutex.lock().await;
        if let Some(mutex_vec) = trades.get(&payload.symbol) {
            let mut trade_list = mutex_vec.lock().await;

            // 查找匹配的交易
            if let Some(index) = trade_list.iter().position(|trade| trade.id == payload.id) {
                let trade = trade_list.remove(index);
                let (side, position_side, price) = match trade.direction {
                    TradeDirection::Long => ("SELL", "LONG", book.1.clone()),
                    TradeDirection::Short => ("BUY", "SHORT", book.0.clone()),
                };

                let order_response = create_order(
                    &payload.symbol,
                    side,
                    position_side,
                    "MARKET",        // 假设使用市价单
                    &trade.quantity, // 将数量格式化为字符串
                    None,            // 市价单无需价格
                    None,            // 此示例未设置止损价格
                )
                .await;

                match order_response {
                    Ok(order) => match get_order(&payload.symbol, order.orderId).await {
                        Ok(b_order) => {
                            create_trade_record(&database, &trade, &b_order.avgPrice).await;

                            // 返回平仓结果
                            let result = CloseTradeResponse {
                                id: trade.id,
                                symbol: payload.symbol,
                                direction: trade.direction,
                                entry_price: trade.entry_price,
                                close_price: b_order.avgPrice,
                                quantity: trade.quantity,
                            };

                            return (StatusCode::OK, Json(result)).into_response();
                        }
                        Err(_) => {
                            return AppError::new(StatusCode::BAD_REQUEST, "Trade not found")
                                .into_response()
                        }
                    },
                    Err(_) => {
                        return AppError::new(StatusCode::BAD_REQUEST, "Trade not found")
                            .into_response()
                    }
                }
            } else {
                return AppError::new(StatusCode::BAD_REQUEST, "Trade not found").into_response();
            }
        } else {
            AppError::new(StatusCode::BAD_REQUEST, "Symbol not found").into_response()
        }
    } else {
        AppError::new(StatusCode::BAD_REQUEST, "failed, symbol").into_response()
    }
}

pub async fn get_all_history_trades(
    Extension(database): Extension<DatabaseConnection>,
    Query(params): Query<TradeQueryParams>,
) -> impl IntoResponse {
    use sea_orm::QueryOrder;

    let mut query = trades::Entity::find();

    // 按 symbol 查询
    if let Some(symbol) = &params.symbol {
        query = query.filter(trades::Column::Symbol.eq(symbol.as_str()));
    }

    // 按时间范围查询
    if let Some(start_time) = params.start_time {
        query = query.filter(trades::Column::CreatedAt.gte(start_time));
    }
    if let Some(end_time) = params.end_time {
        query = query.filter(trades::Column::CreatedAt.lte(end_time));
    }

    // 执行查询并返回结果
    match query
        .order_by_desc(trades::Column::CreatedAt)
        .all(&database)
        .await
    {
        Ok(result) => (StatusCode::OK, Json(result)).into_response(),
        Err(err) => {
            let error_message = format!("Failed to fetch trades: {}", err);
            AppError::new(StatusCode::INTERNAL_SERVER_ERROR, error_message).into_response()
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct DeleteResponse {
    pub id: u32,
}

pub async fn delete_trade_by_id(
    Extension(database): Extension<DatabaseConnection>,
    Query(params): Query<DeleteResponse>,
) -> impl IntoResponse {
    let id = params.id;
    // 按 ID 删除
    match trades::Entity::delete_by_id(id).exec(&database).await {
        Ok(delete_result) => {
            if delete_result.rows_affected > 0 {
                // 删除成功的响应
                let response = DeleteResponse { id };
                (StatusCode::OK, Json(response)).into_response()
            } else {
                // 没有找到要删除的记录
                let error_message = format!("No trade found with id: {}", id);
                AppError::new(StatusCode::NOT_FOUND, error_message).into_response()
            }
        }
        Err(err) => {
            // 处理数据库操作错误
            let error_message = format!("Failed to delete trade: {}", err);
            AppError::new(StatusCode::INTERNAL_SERVER_ERROR, error_message).into_response()
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct AdjustmentRequest {
    pub id: u8,
}

pub async fn get_adjustments(
    Extension(adjustments): Extension<Arc<HashMap<u8, Mutex<AdjustmentConfig>>>>,
    Query(params): Query<AdjustmentRequest>,
) -> impl IntoResponse {
    let id = params.id;
    if let Some(mutex_config) = adjustments.get(&id) {
        // 尝试解锁 Mutex
        let config = mutex_config.lock().await;
        let adjustment = config.adjustments.clone();
        (StatusCode::OK, Json(adjustment)).into_response()
    } else {
        // 如果键不存在，返回 404
        let error_message = format!("Failed to get adjustment id: {}", id);
        AppError::new(StatusCode::INTERNAL_SERVER_ERROR, error_message).into_response()
    }
}

#[derive(Deserialize)]
pub struct UpdateAdjustmentRequest {
    pub id: u8,
    pub adjustment: Vec<Adjustment>,
}

pub async fn update_adjustments(
    Extension(adjustments): Extension<Arc<HashMap<u8, Mutex<AdjustmentConfig>>>>,
    Json(payload): Json<UpdateAdjustmentRequest>,
) -> impl IntoResponse {
    let id = payload.id;
    if let Some(mutex_config) = adjustments.get(&id) {
        // 尝试解锁 Mutex
        let mut config = mutex_config.lock().await;
        config.adjustments = payload.adjustment;
        let response = AdjustmentRequest { id };
        (StatusCode::OK, Json(response)).into_response()
    } else {
        // 如果键不存在，返回 404
        let error_message = format!("Failed to get adjustment id: {}", id);
        AppError::new(StatusCode::INTERNAL_SERVER_ERROR, error_message).into_response()
    }
}

pub async fn get_user_hold() -> impl IntoResponse {
    let data = get_risk().await.unwrap();
    (StatusCode::OK, Json(data).into_response())
}
