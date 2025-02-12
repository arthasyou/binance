pub mod auth_model;
pub mod trade_model;

use sea_orm::prelude::DateTimeWithTimeZone;
use serde::{Deserialize, Serialize};

pub trait IntoCommonResponse {
    fn into_common_response_data(self) -> CommonResponse;
}

impl<T> IntoCommonResponse for T
where
    T: Serialize,
{
    fn into_common_response_data(self) -> CommonResponse {
        CommonResponse {
            code: 0,
            data: serde_json::to_value(self).expect("Failed to convert to serde_json::Value"),
            message: String::from("Success"),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct CommonResponse {
    pub code: u16,
    pub data: serde_json::Value,
    pub message: String,
}

impl Default for CommonResponse {
    fn default() -> Self {
        CommonResponse {
            code: 0,
            data: serde_json::Value::Null,
            message: String::from("Success"),
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct CommonParams {
    pub skip: Option<u64>,  // 允许为 None，且当存在时必须为非负数
    pub limit: Option<u64>, // 允许为 None，且当存在时必须为非负数
    pub start_time: Option<DateTimeWithTimeZone>, // 允许为 None，且当存在时必须为非负数
    pub end_time: Option<DateTimeWithTimeZone>, // 允许为 None，且当存在时必须为非负数
}
