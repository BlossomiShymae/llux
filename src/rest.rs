use irelia::{rest::LCUClient, LCUError};
use serde_json::Value;

use crate::cli::RequestMethod;

pub async fn send_request(
    client: &LCUClient<'_>,
    method: RequestMethod,
    path: &str,
    body: Option<Value>,
) -> Result<Option<Value>, LCUError> {
    match method {
        RequestMethod::Get => client.get::<Value>(path).await,
        RequestMethod::Delete => client.delete::<Value>(path).await,
        RequestMethod::Head => client.head::<Value>(path).await,
        RequestMethod::Post => client.post::<Value, Value>(path, body).await,
        RequestMethod::Put => client.put::<Value, Value>(path, body).await,
        RequestMethod::Patch => client.patch::<Value, Value>(path, body).await,
    }
}
