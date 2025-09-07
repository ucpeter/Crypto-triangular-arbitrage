use crate::models::PairPrice;
use reqwest::Client;
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::error::Error;
use tracing::info;

/// ---------------- Binance ----------------
async fn fetch_binance(client: &Client) -> Result<Vec<PairPrice>, String> {
    let info_url = "https://api.binance.com/api/v3/exchangeInfo";
    let info: Value = client.get(info_url).send().await.map_err(|e| e.to_string())?.json().await.map_err(|e| e.to_string())?;
    let mut symbol_map: HashMap<String, (String, String)> = HashMap::new();

    if let Some(arr) = info["symbols"].as_array() {
        for obj in arr {
            if obj["status"] == "TRADING" && obj["isSpotTradingAllowed"] == true {
                if let (Some(base), Some(quote), Some(symbol)) =
                    (obj["baseAsset"].as_str(), obj["quoteAsset"].as_str(), obj["symbol"].as_str())
                {
                    symbol_map.insert(
                        symbol.to_uppercase(),
                        (base.to_uppercase(), quote.to_uppercase()),
                    );
                }
            }
        }
    }

    let price_url = "https://api.binance.com/api/v3/ticker/price";
    let prices: Value = client.get(price_url).send().await.map_err(|e| e.to_string())?.json().await.map_err(|e| e.to_string())?;

    let mut out = Vec::new();
    if let Some(arr) = prices.as_array() {
        for obj in arr {
            if let (Some(symbol), Some(price_str)) = (obj["symbol"].as_str(), obj["price"].as_str()) {
                let symbol = symbol.to_uppercase();
                if let Some((base, quote)) = symbol_map.get(&symbol) {
                    if let Ok(price) = price_str.parse::<f64>() {
                        if price > 0.0 {
                            out.push(PairPrice {
                                base: base.clone(),
                                quote: quote.clone(),
                                price,
                                is_spot: true,
                            });
                        }
                    }
                }
            }
        }
    }

    info!("binance returned {} pairs", out.len());
    Ok(out)
}

/// ---------------- KuCoin ----------------
async fn fetch_kucoin(client: &Client) -> Result<Vec<PairPrice>, String> {
    let sym_url = "https://api.kucoin.com/api/v1/symbols";
    let sym_json: Value = client.get(sym_url).send().await.map_err(|e| e.to_string())?.json().await.map_err(|e| e.to_string())?;

    let mut tradable: HashSet<String> = HashSet::new();
    if let Some(arr) = sym_json["data"].as_array() {
        for s in arr {
            if s["enableTrading"] == true {
                if let Some(sym) = s["symbol"].as_str() {
                    tradable.insert(sym.to_string());
                }
            }
        }
    }

    let url = "https://api.kucoin.com/api/v1/market/allTickers";
    let resp: Value = client.get(url).send().await.map_err(|e| e.to_string())?.json().await.map_err(|e| e.to_string())?;

    let mut out = Vec::new();
    if let Some(arr) = resp["data"]["ticker"].as_array() {
        for obj in arr {
            if let (Some(symbol), Some(price_str)) = (obj["symbol"].as_str(), obj["last"].as_str()) {
                if !tradable.contains(symbol) { continue; }
                if let Some((base, quote)) = symbol.split_once('-') {
                    if let Ok(price) = price_str.parse::<f64>() {
                        if price > 0.0 {
                            out.push(PairPrice {
                                base: base.to_string(),
                                quote: quote.to_string(),
                                price,
                                is_spot: true,
                            });
                        }
                    }
                }
            }
        }
    }

    info!("kucoin returned {} pairs", out.len());
    Ok(out)
}

    // ----------------- BYBIT -----------------
pub async fn fetch_bybit(client: &Client) -> Result<Vec<PairPrice>, String> {
    info!("fetching bybit");

    // Step 1: fetch instruments meta (only keep status == "Trading")
    let info_url = "https://api.bybit.com/v5/market/instruments-info?category=spot";
    let info: serde_json::Value = client
        .get(info_url)
        .send()
        .await
        .map_err(|e| format!("bybit info http error: {}", e))?
        .json()
        .await
        .map_err(|e| format!("bybit info decode error: {}", e))?;

    let mut active_map: std::collections::HashMap<String, (String, String)> =
        std::collections::HashMap::new();
    if let Some(list) = info["result"]["list"].as_array() {
        for v in list {
            if v["status"].as_str() == Some("Trading") {
                if let (Some(sym), Some(base), Some(quote)) = (
                    v.get("symbol").and_then(|s| s.as_str()),
                    v.get("baseCoin").and_then(|s| s.as_str()),
                    v.get("quoteCoin").and_then(|s| s.as_str()),
                ) {
                    active_map.insert(
                        sym.to_uppercase(),
                        (base.to_uppercase(), quote.to_uppercase()),
                    );
                }
            }
        }
    }
    info!("Bybit metadata loaded {} trading spot symbols", active_map.len());

    // Step 2: fetch tickers and only keep those in active_map
    let tickers_url = "https://api.bybit.com/v5/market/tickers?category=spot";
    let tickers: serde_json::Value = client
        .get(tickers_url)
        .send()
        .await
        .map_err(|e| format!("bybit tickers http error: {}", e))?
        .json()
        .await
        .map_err(|e| format!("bybit tickers decode error: {}", e))?;

    let mut out: Vec<PairPrice> = Vec::new();
    let mut kept = 0usize;
    let mut skipped = 0usize;

    if let Some(list) = tickers["result"]["list"].as_array() {
        for v in list {
            if let (Some(sym), Some(price_s)) =
                (v.get("symbol").and_then(|s| s.as_str()), v.get("lastPrice").and_then(|p| p.as_str()))
            {
                let sym_u = sym.to_uppercase();
                if let Some((base, quote)) = active_map.get(&sym_u) {
                    if let Ok(price) = price_s.parse::<f64>() {
                        if price > 0.0 {
                            out.push(PairPrice {
                                base: base.clone(),
                                quote: quote.clone(),
                                price,
                                is_spot: true,
                            });
                            kept += 1;
                        } else {
                            skipped += 1;
                        }
                    } else {
                        skipped += 1;
                    }
                } else {
                    // not active according to instruments-info
                    skipped += 1;
                }
            }
        }
    }

    info!("Bybit kept {} pairs, skipped {}", kept, skipped);
    Ok(out)
}


// ----------------- GATE.IO -----------------
pub async fn fetch_gateio(client: &Client) -> Result<Vec<PairPrice>, String> {
    info!("fetching gateio");

    // Step 1: fetch currency_pairs metadata (keep trade_status == "tradable")
    let symbols_url = "https://api.gate.io/api/v4/spot/currency_pairs";
    let symbols: Vec<serde_json::Value> = client
        .get(symbols_url)
        .send()
        .await
        .map_err(|e| format!("gateio symbols http error: {}", e))?
        .json()
        .await
        .map_err(|e| format!("gateio symbols decode error: {}", e))?;

    let mut tradable_map: std::collections::HashMap<String, (String, String)> =
        std::collections::HashMap::new();
    for s in symbols {
        if s["trade_status"].as_str() == Some("tradable") {
            if let (Some(id), Some(base), Some(quote)) = (
                s.get("id").and_then(|x| x.as_str()),
                s.get("base").and_then(|x| x.as_str()),
                s.get("quote").and_then(|x| x.as_str()),
            ) {
                tradable_map.insert(id.to_uppercase(), (base.to_uppercase(), quote.to_uppercase()));
            }
        }
    }
    info!("Gate.io metadata loaded {} tradable pairs", tradable_map.len());

    // Step 2: fetch tickers and only keep those in tradable_map
    let tickers_url = "https://api.gate.io/api/v4/spot/tickers";
    let tickers: Vec<serde_json::Value> = client
        .get(tickers_url)
        .send()
        .await
        .map_err(|e| format!("gateio tickers http error: {}", e))?
        .json()
        .await
        .map_err(|e| format!("gateio tickers decode error: {}", e))?;

    let mut out: Vec<PairPrice> = Vec::new();
    let mut kept = 0usize;
    let mut skipped = 0usize;

    for t in tickers {
        if let (Some(sym), Some(price_s)) = (t.get("currency_pair").and_then(|s| s.as_str()), t.get("last").and_then(|p| p.as_str())) {
            let sym_u = sym.to_uppercase();
            if let Some((base, quote)) = tradable_map.get(&sym_u) {
                if let Ok(price) = price_s.parse::<f64>() {
                    if price > 0.0 {
                        out.push(PairPrice {
                            base: base.clone(),
                            quote: quote.clone(),
                            price,
                            is_spot: true,
                        });
                        kept += 1;
                    } else {
                        skipped += 1;
                    }
                } else {
                    skipped += 1;
                }
            } else {
                // ticker exists but not marked tradable in metadata
                skipped += 1;
            }
        }
    }

    info!("Gate.io kept {} pairs, skipped {}", kept, skipped);
    Ok(out)
                }
/// ---------------- Dispatcher ----------------
pub async fn fetch_exchange_data(exchange: &str) -> Result<Vec<PairPrice>, String> {
    let client = Client::new();
    match exchange {
        "binance" => fetch_binance(&client).await,
        "kucoin" => fetch_kucoin(&client).await,
        "bybit" => fetch_bybit(&client).await,
        "gate" | "gateio" => fetch_gateio(&client).await,
        _ => Err(format!("unsupported exchange: {}", exchange)),
    }
                }
