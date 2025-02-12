mod binance;
mod db;
mod error;
mod handlers;
mod models;
mod mw;
mod orm;
mod routes;
mod trade;
mod utils;
mod websocket_lib;

use binance::leverage::get_quantity_precision;
use db::connect_db;
use dotenvy::dotenv;
use futures_util::future::join_all;
use sea_orm::DatabaseConnection;
use std::{collections::HashMap, env, sync::Arc};
use trade::{Adjustment, AdjustmentConfig, Trade};
use utils::{create_adjustment_config_raw, TradeIdGenerator};

use service_utils_rs::{services::jwt::Jwt, settings::Settings};
use tokio::{self, sync::Mutex};
use websocket_lib::connection::connect_to_websocket;

#[tokio::main]
async fn main() {
    dotenv().ok();
    let settings = Settings::new("config/services.toml").unwrap();
    let jwt = Jwt::new(settings.jwt);
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let database = connect_db(&database_url).await.unwrap();
    let port = env::var("PORT").expect("PORT must be set");
    let base_symbols = vec![
        "ada", "crv", "doge", "dot", "hbar", "om", "xlm", "xrp", "sui", "wif", "render", "neiro",
        "pnut", "act", "ltc", "trx", "bnb", "wld", "fil",
    ];

    // let base_symbols = vec![
    //     "ADA", "CRV", "DOGE", "DOT", "HBAR", "OM", "SHIB", "XLM", "XRP", "SOL", "SUI", "WIF",
    //     "RENDER", "NEIRO", "PNUT", "ACT", "USUAL", "LTC", "TRX", "BNB", "WLD", "FIL",
    // ];

    let symbols: Vec<String> = base_symbols.iter().map(|s| format!("{}usdt", s)).collect();

    let precisions = get_quantity_precision(&symbols).await.unwrap();
    println!("{:?}", &precisions);

    // 初始化共享状态
    let trades = init_trade(&symbols);
    let prices = init_price(&symbols);
    let id_generator = Arc::new(TradeIdGenerator::new());
    let adjustment = init_adjustment();

    let ws_task = start_websocket(&symbols, trades.clone(), prices.clone(), database.clone());

    let routes = routes::create_routes(
        trades.clone(),
        prices.clone(),
        id_generator.clone(),
        database,
        Arc::new(precisions),
        adjustment,
        jwt,
    );

    let addr = format!("0.0.0.0:{}", port);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

    let http_task = axum::serve(listener, routes);
    let _ = tokio::join!(ws_task, http_task);
}

// WebSocket 启动函数
async fn start_websocket(
    symbles: &Vec<String>,
    trades: Arc<HashMap<String, Mutex<Vec<Trade>>>>,
    prices: Arc<HashMap<String, Mutex<(String, String)>>>,
    database: DatabaseConnection,
) {
    let mut tasks: Vec<tokio::task::JoinHandle<()>> = Vec::new();

    for symbol in symbles {
        let symbol_clone = symbol.clone();
        let trades_clone = trades.clone();
        let prices_clone = prices.clone();
        let database_clone = database.clone();
        let task = tokio::spawn(async move {
            connect_to_websocket(symbol_clone, trades_clone, prices_clone, database_clone).await;
        });
        tasks.push(task);
    }

    // 使用 join_all 处理所有 WebSocket 任务
    join_all(tasks).await;
}

fn init_trade(symbols: &Vec<String>) -> Arc<HashMap<String, Mutex<Vec<Trade>>>> {
    let map = symbols
        .iter()
        .map(|symbol| (symbol.clone(), Mutex::new(Vec::new())))
        .collect::<HashMap<_, _>>();

    Arc::new(map)
}

fn init_price(symbols: &Vec<String>) -> Arc<HashMap<String, Mutex<(String, String)>>> {
    // let r = get_quantity_precision(symbols).await.unwrap();
    let map = symbols
        .iter()
        .map(|symbol| {
            (
                symbol.clone(),
                Mutex::new(("0".to_string(), "0".to_string())),
            )
        })
        .collect::<HashMap<_, _>>();

    Arc::new(map)
}

fn init_adjustment() -> Arc<HashMap<u8, Mutex<AdjustmentConfig>>> {
    let mut map = HashMap::new();
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
            min: 0.8,
            max: Some(0.89),
            adjustment: 0.64,
        },
        Adjustment {
            min: 0.9,
            max: Some(0.99),
            adjustment: 0.81,
        },
        Adjustment {
            min: 1.0,
            max: Some(1.1),
            adjustment: 0.90,
        },
    ];

    let a = create_adjustment_config_raw(adjustment.clone());
    let b = create_adjustment_config_raw(adjustment.clone());

    map.insert(1, Mutex::new(a));
    map.insert(2, Mutex::new(b));

    Arc::new(map)
}
