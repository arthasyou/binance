// mod account;
pub mod account;
pub mod leverage;
pub mod order;
pub mod record_api;

use crate::error::{Error, Result};
use hmac::{Hmac, Mac};
use lazy_static::lazy_static;
use reqwest::{Client, Method};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use sha2::Sha256;
use std::{
    env,
    time::{SystemTime, UNIX_EPOCH},
};

type HmacSha256 = Hmac<Sha256>;

// 使用 lazy_static 来确保只初始化一次
lazy_static! {
    pub static ref API_KEY: String = env::var("API_KEY").expect("API_KEY must be set in .env");
    pub static ref API_SECRET: String =
        env::var("API_SECRET").expect("API_SECRET must be set in .env");
}

// const API_KEY: &str = "8a6bd7a5be0e9c77a090670921adbbc4cd4101f070d993f2b7378eab01749a54";
// const API_SECRET: &str = "33089124cde387d199ceab2b469cc3939d9df3f6f42278052340c6880a8e0e31";
// const BASE_URL: &str = "https://testnet.binancefuture.com";
// const API_KEY: &str = "jIeoG4sLpJp8Sba8MJyKKLc46CzlRTHMMwXkgoVPkXm8DwcHaCNHLse8rpKwnAgx";
// const API_SECRET: &str = "XsYec95qcxyou2XaXvYrT9KsgUVSgS3DLGKM0jzurtHrODpYUmHz7LCh6p4nZZNd";
const BASE_URL: &str = "https://fapi.binance.com";

fn create_signature(secret: &str, query_string: &str) -> String {
    let mut mac =
        HmacSha256::new_from_slice(secret.as_bytes()).expect("HMAC can take key of any size");
    mac.update(query_string.as_bytes());
    hex::encode(mac.finalize().into_bytes())
}

pub async fn request<T: DeserializeOwned>(url: &str, method: Method, api_key: &str) -> Result<T> {
    let client = Client::new();

    // 根据方法构造请求
    let request_builder = match method {
        Method::GET => client.get(url),
        Method::POST => client.post(url),
        Method::PUT => client.put(url),
        Method::DELETE => client.delete(url),
        _ => return Err(Error::SystemError("unknow method".to_string())), // 如果有其他方法，返回错误
    };

    // 添加通用头部
    let request_builder = request_builder.header("X-MBX-APIKEY", api_key);

    // 发送请求并获取响应
    let response_text = request_builder.send().await?.text().await?;

    // 打印响应内容
    println!("Response content: {}", response_text);

    // 解析响应为指定类型
    let response = serde_json::from_str::<T>(&response_text).map_err(|e| {
        println!("Response parse error: {:?}", response_text);
        e
    })?;

    Ok(response)
}

fn create_timestamp() -> String {
    // 获取当前时间戳
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_millis()
        .to_string()
}
