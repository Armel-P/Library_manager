pub mod structs;

use reqwest::{header::{HeaderMap, HeaderName, HeaderValue}, Client, Method};
use serde::{de::DeserializeOwned, Serialize};
use std::sync::mpsc;
use crate::constants::URL;

#[derive(serde::Deserialize)]
struct ErrorResponse {
    error: String,
}

async fn api_request<B, T>(
    method: Method,
    endpoint: &str,
    body: Option<&B>,
    headers: Option<&[(&str, &str)]>,
) -> Result<T, String>
where
    B: Serialize,
    T: DeserializeOwned,
{
    let client = Client::new();
    let mut req = client.request(method, format!("{URL}{endpoint}"));

    if let Some(b) = body {
        req = req.json(b);
    }

    if let Some(hs) = headers {
        let mut map = HeaderMap::new();
        for (k, v) in hs {
            let name: HeaderName = (*k).parse().map_err(|e| format!("{e}"))?;
            let value: HeaderValue = (*v).parse().map_err(|e| format!("{e}"))?;
            map.insert(name, value);
        }
        req = req.headers(map);
    }

    let res = req.send().await.map_err(|e| format!("Request failed: {e}"))?;

    if res.status().is_success() {
        res.json::<T>().await.map_err(|e| format!("Invalid response: {e}"))
    } else {
        match res.json::<ErrorResponse>().await {
            Ok(err) => Err(err.error),
            Err(e) => Err(format!("Error parsing error: {e}")),
        }
    }
}

pub fn spawn_request<B, T>(
    method: Method,
    endpoint: String,
    body: Option<B>,
    headers: Option<Vec<(String, String)>>,
) -> mpsc::Receiver<Result<T, String>>
where
    B: Serialize + Send + 'static,
    T: DeserializeOwned + Send + 'static,
{
    let (tx, rx) = mpsc::channel();

    std::thread::spawn(move || {
        let rt = match tokio::runtime::Runtime::new() {
            Ok(rt) => rt,
            Err(e) => {
                let _ = tx.send(Err(format!("Failed to start async runtime: {e}")));
                return;
            }
        };
        rt.block_on(async move {
            let header_refs: Option<Vec<(&str, &str)>> = headers
                .as_ref()
                .map(|hs| hs.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect());

            let result = api_request::<B, T>(
                method,
                &endpoint,
                body.as_ref(),
                header_refs.as_deref(),
            )
            .await;

            let _ = tx.send(result);
        });
    });

    rx
}
