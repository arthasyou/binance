use crate::{
    trade::Trade,
    utils::{self, format_url, trim_trailing_zeros},
};
use futures_util::{SinkExt, StreamExt};
use sea_orm::DatabaseConnection;
use std::{collections::HashMap, sync::Arc};
use tokio::{
    sync::Mutex,
    time::{self, timeout, Duration},
};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use url::Url;

pub async fn connect_to_websocket(
    symbol: String,
    trades: Arc<HashMap<String, Mutex<Vec<Trade>>>>,
    prices: Arc<HashMap<String, Mutex<(String, String)>>>,
    database: DatabaseConnection,
) {
    let url = format_url(&symbol);
    // let key = symbol.to_string();
    loop {
        match connect_async(Url::parse(&url).unwrap()).await {
            Ok((mut socket, _response)) => {
                loop {
                    let msg = timeout(Duration::from_secs(30), socket.next()).await;
                    match msg {
                        Ok(Some(inner_msg)) => match inner_msg {
                            Ok(Message::Text(text)) => match utils::parse_trade_json(&text) {
                                Ok(data) => {
                                    // println!("{:?}", data);
                                    let book_price = (
                                        trim_trailing_zeros(&data.a),
                                        trim_trailing_zeros(&data.b),
                                    );
                                    if let Some(mutex_f64) = prices.get(&symbol) {
                                        let mut book = mutex_f64.lock().await;
                                        *book = book_price.clone();
                                    } else {
                                        // eprintln!("failed symbol: {:?}", data.s);
                                    }

                                    // Access the `Mutex<Vec<Trade>>` for the given key
                                    if let Some(mutex_vec) = trades.get(&symbol) {
                                        let mut vec = mutex_vec.lock().await;

                                        vec.retain(|t| {
                                            if t.is_closed {
                                                false // 如果 t.is_closed 为 true，则从 vec 中移除
                                            } else {
                                                true // 保留元素，并在后续的 for 循环中处理
                                            }
                                        });

                                        // Update the price for each trade in the vector
                                        for t in vec.iter_mut() {
                                            t.update_price(book_price.clone(), &database).await;
                                        }

                                        // Alternatively, you may want to add a new trade instead
                                        // vec.push(trade);
                                    } else {
                                        // eprintln!("failed symbol: {:?}", data.s);
                                    }
                                }
                                Err(_e) => {
                                    // println!("price: {}", text);
                                    // eprintln!("failed to parse JSON: {:?}", e);
                                }
                            },
                            Ok(Message::Ping(ping)) => {
                                // println!("Received Ping from {}: {:?}", url, ping);
                                socket
                                    .send(Message::Pong(ping))
                                    .await
                                    .expect("Failed to send Pong");
                            }
                            Ok(Message::Close(_frame)) => {
                                // println!("Connection closed from {}: {:?}", url, frame);
                                break;
                            }
                            _ => (),
                        },
                        Ok(None) => {
                            // println!("WebSocket closed, reconnecting...");
                            break;
                        }
                        Err(_) => {
                            // eprintln!(
                            //     "Timeout while waiting for WebSocket message, reconnecting..."
                            // );
                            break;
                        }
                    }
                }

                // Close the connection

                // socket
                //     .close(None)
                //     .await
                //     .expect("Failed to close connection");
            }
            Err(e) => {
                eprintln!(
                    "Connection failed to {}: {:?}. Retrying in 5 seconds...",
                    url, e
                );
            }
        }

        // Wait and retry connection after 5 seconds
        time::sleep(Duration::from_secs(5)).await;
        println!("Reconnecting to {}...", url);
    }
}
