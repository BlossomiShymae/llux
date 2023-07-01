use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WebSocketMessage {
    pub opcode: i64,
    pub event: String,
    pub data: Value,
    pub uri: String,
    pub event_type: String,
    pub timestamp: u128,
}

impl WebSocketMessage {
    pub fn from_value(v: Value) -> WebSocketMessage {
        let message = v.as_array().unwrap();
        let opcode = message.get(0).unwrap().as_i64().unwrap();
        let event: String = message.get(1).unwrap().as_str().unwrap().into();
        let payload: WebSocketPayload =
            serde_json::from_value(message.get(2).unwrap().clone()).unwrap();
        WebSocketMessage {
            opcode,
            event,
            data: payload.data,
            uri: payload.uri,
            event_type: payload.event_type,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebSocketPayload {
    pub data: Value,
    pub uri: String,
    pub event_type: String,
}
