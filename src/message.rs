use serde::{Serialize, Deserialize};
use serde_json::Value;

/// 请求消息
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Request {
    /// 请求ID
    pub id: i64,
    /// 方法名称
    pub method: String,
    /// 可选参数
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
}

/// 响应消息
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Response<T = Value> {
    /// 请求ID
    pub id: i64,
    /// 响应结果
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<T>,
    /// 错误信息
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<Value>,
}

/// 事件消息
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Event {
    /// 方法名称
    pub method: String,
    /// 事件参数
    #[serde(default)]
    pub params: Option<Value>,
    /// 会话ID（可选）
    #[serde(rename = "sessionId")]
    pub session_id: Option<String>,
} 