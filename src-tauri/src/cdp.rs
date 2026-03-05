use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::Duration;
use tokio::time::sleep;
use futures_util::sink::SinkExt;
use futures_util::stream::StreamExt;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::protocol::Message;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CdpMetrics {
    pub dns_duration_ms: u64,
    pub tcp_duration_ms: u64,
    pub first_byte_ms: u64,
    pub dom_ready_ms: u64,
    pub page_load_ms: u64,
}

#[derive(Deserialize)]
struct CdpTarget {
    #[serde(rename = "webSocketDebuggerUrl")]
    websocket_url: String,
    #[serde(rename = "type")]
    target_type: String,
}

pub async fn fetch_real_metrics(port: u16) -> Option<CdpMetrics> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(2))
        .build()
        .ok()?;

    let json_url = format!("http://127.0.0.1:{}/json", port);
    
    // Poll for targets to be available
    let mut targets: Vec<CdpTarget> = Vec::new();
    for _ in 0..10 {
        if let Ok(resp) = client.get(&json_url).send().await {
            if let Ok(t) = resp.json::<Vec<CdpTarget>>().await {
                targets = t.into_iter().filter(|t| t.target_type == "page").collect();
                if !targets.is_empty() {
                    break;
                }
            }
        }
        sleep(Duration::from_millis(500)).await;
    }

    if targets.is_empty() {
        return None;
    }

    let ws_url = &targets[0].websocket_url;
    let (mut ws_stream, _) = connect_async(ws_url).await.ok()?;

    // Enable Runtime
    let enable_msg = serde_json::json!({
        "id": 1,
        "method": "Runtime.enable",
        "params": {}
    });
    ws_stream.send(Message::Text(enable_msg.to_string().into())).await.ok()?;

    // Wait for page to load and get timing
    // We try multiple times because the page might still be loading
    let mut metrics = None;
    for i in 0..15 {
        let eval_msg = serde_json::json!({
            "id": i + 100,
            "method": "Runtime.evaluate",
            "params": {
                "expression": "JSON.stringify(window.performance.getEntriesByType('navigation')[0] || window.performance.timing)",
                "returnByValue": true
            }
        });

        if ws_stream.send(Message::Text(eval_msg.to_string().into())).await.is_err() {
            break;
        }

        while let Some(Ok(msg)) = ws_stream.next().await {
            if let Message::Text(text) = msg {
                if let Ok(v) = serde_json::from_str::<Value>(&text) {
                    if v["id"].as_u64() == Some((i + 100) as u64) {
                        if let Some(result_str) = v["result"]["result"]["value"].as_str() {
                            if let Ok(timing) = serde_json::from_str::<Value>(result_str) {
                                // For `performance.timing` (deprecated), `loadEventEnd` is an absolute epoch time.
                                // For `PerformanceNavigationTiming` (modern), `loadEventEnd` is a high-res relative float.
                                let load_event_end = timing["loadEventEnd"].as_f64().unwrap_or(0.0);
                                if load_event_end > 0.0 {
                                    metrics = Some(parse_timing(timing));
                                    break;
                                }
                            }
                        }
                        break;
                    }
                }
            }
        }

        if metrics.is_some() {
            break;
        }
        sleep(Duration::from_millis(1000)).await;
    }

    metrics
}

fn parse_timing(t: Value) -> CdpMetrics {
    // We check if the response is from modern `PerformanceNavigationTiming` or old `performance.timing`.
    // The modern API uses float milliseconds relative to fetchStart (which is usually around 0.0).
    // The deprecated API uses u64 absolute timestamps.
    let is_modern = t.get("name").is_some();
    
    let get = |k: &str| t[k].as_f64().unwrap_or(0.0);

    let fetch_start = get("fetchStart");
    let dns_start = get("domainLookupStart");
    let dns_end = get("domainLookupEnd");
    let connect_start = get("connectStart");
    let connect_end = get("connectEnd");
    let request_start = get("requestStart");
    let response_start = get("responseStart");
    let dom_content = get("domContentLoadedEventEnd");
    let load_event = get("loadEventEnd");

    if is_modern {
        CdpMetrics {
            dns_duration_ms: if dns_end >= dns_start { (dns_end - dns_start) as u64 } else { 0 },
            tcp_duration_ms: if connect_end >= connect_start { (connect_end - connect_start) as u64 } else { 0 },
            first_byte_ms: if response_start >= request_start { (response_start - request_start) as u64 } else { 0 },
            dom_ready_ms: if dom_content >= 0.0 { dom_content as u64 } else { 0 },
            page_load_ms: if load_event >= 0.0 { load_event as u64 } else { 0 },
        }
    } else {
        CdpMetrics {
            dns_duration_ms: if dns_end >= dns_start { (dns_end - dns_start) as u64 } else { 0 },
            tcp_duration_ms: if connect_end >= connect_start { (connect_end - connect_start) as u64 } else { 0 },
            first_byte_ms: if response_start >= request_start { (response_start - request_start) as u64 } else { 0 },
            dom_ready_ms: if dom_content >= fetch_start { (dom_content - fetch_start) as u64 } else { 0 },
            page_load_ms: if load_event >= fetch_start { (load_event - fetch_start) as u64 } else { 0 },
        }
    }
}
