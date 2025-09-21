use crate::models::PairPrice;
use futures_util::{StreamExt, SinkExt};
use serde_json::Value;
use std::collections::HashMap;
use tokio_tungstenite::connect_async;
use tracing::info;

/// ---------------- Binance WS (on-demand) ----------------
pub async fn fetch_binance_ws() -> Result<Vec<PairPrice>, String> {
    let url = "wss://stream.binance.com:9443/ws/!ticker@arr";
    let (ws_stream, _) = connect_async(url)
        .await
        .map_err(|e| format!("binance ws connect error: {}", e))?;
    let (mut _write, mut read) = ws_stream.split();

    // Read a single snapshot message
    let msg = match read.next().await {
        Some(Ok(m)) => m,
        Some(Err(e)) => return Err(format!("binance ws read error: {}", e)),
        None => return Err("binance ws closed early".to_string()),
    };

    let text = msg.into_text().map_err(|e| e.to_string())?;
    let arr: Value = serde_json::from_str(&text).map_err(|e| e.to_string())?;

    // Map symbol info from REST exchangeInfo
    let info_url = "https://api.binance.com/api/v3/exchangeInfo";
    let info: Value = reqwest::get(info_url).await.map_err(|e| e.to_string())?.json().await.map_err(|e| e.to_string())?;
    let mut symbol_map: HashMap<String, (String, String)> = HashMap::new();

    if let Some(symbols) = info["symbols"].as_array() {
        for obj in symbols {
            if obj["status"] == "TRADING" && obj["isSpotTradingAllowed"] == true {
                if let (Some(base), Some(quote), Some(symbol)) =
                    (obj["baseAsset"].as_str(), obj["quoteAsset"].as_str(), obj["symbol"].as_str())
                {
                    symbol_map.insert(symbol.to_uppercase(), (base.to_uppercase(), quote.to_uppercase()));
                }
            }
        }
    }

    // Convert snapshot into PairPrice
    let mut out = Vec::new();
    if let Some(tickers) = arr.as_array() {
        for obj in tickers {
            if let (Some(symbol), Some(price_str), Some(vol_str)) =
                (obj["s"].as_str(), obj["c"].as_str(), obj["q"].as_str()) // ✅ s=symbol, c=lastPrice, q=quoteVolume
            {
                let symbol = symbol.to_uppercase();
                if let Some((base, quote)) = symbol_map.get(&symbol) {
                    if let (Ok(price), Ok(vol)) = (price_str.parse::<f64>(), vol_str.parse::<f64>()) {
                        if price > 0.0 && vol > 0.0 {
                            out.push(PairPrice {
                                base: base.clone(),
                                quote: quote.clone(),
                                price,
                                is_spot: true,
                                liquidity: vol, // ✅ same field as REST
                            });
                        }
                    }
                }
            }
        }
    }

    info!("binance (ws) returned {} pairs", out.len());
    Ok(out)
                                      }
