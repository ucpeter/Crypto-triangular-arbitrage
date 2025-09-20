// src/exchange.rs
use crate::models::PairPrice;
use reqwest::Client;
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use tracing::info;

/// ---------------- Binance ----------------
async fn fetch_binance(client: &Client) -> Result<Vec<PairPrice>, String> {
    let info_url = "https://api.binance.com/api/v3/exchangeInfo";
    let info: Value = client
        .get(info_url)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())?;
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

    // Use 24hr ticker to get price + base/quote volumes
    let price_url = "https://api.binance.com/api/v3/ticker/24hr";
    let tickers: Value = client
        .get(price_url)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())?;

    let mut out = Vec::new();
    if let Some(arr) = tickers.as_array() {
        for obj in arr {
            let sym = obj["symbol"].as_str().unwrap_or("");
            let sym_up = sym.to_uppercase();
            if let Some((base, quote)) = symbol_map.get(&sym_up) {
                // lastPrice, volume (base), quoteVolume
                let price = obj["lastPrice"].as_str().and_then(|s| s.parse::<f64>().ok())
                    .or_else(|| obj["lastPrice"].as_f64())
                    .unwrap_or(0.0);
                let base_vol = obj["volume"].as_str().and_then(|s| s.parse::<f64>().ok())
                    .or_else(|| obj["volume"].as_f64())
                    .unwrap_or(0.0);
                let quote_vol = obj["quoteVolume"].as_str().and_then(|s| s.parse::<f64>().ok())
                    .or_else(|| obj["quoteVolume"].as_f64())
                    .unwrap_or(0.0);

                if price > 0.0 {
                    out.push(PairPrice {
                        base: base.clone(),
                        quote: quote.clone(),
                        price,
                        is_spot: true,
                        base_volume: base_vol,
                        quote_volume: quote_vol,
                    });
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
    let sym_json: Value = client
        .get(sym_url)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())?;

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
    let resp: Value = client
        .get(url)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())?;

    let mut out = Vec::new();
    if let Some(arr) = resp["data"]["ticker"].as_array() {
        for obj in arr {
            let symbol = obj["symbol"].as_str().unwrap_or("");
            if !tradable.contains(symbol) {
                continue;
            }
            if let Some((base, quote)) = symbol.split_once('-') {
                let price = obj["last"].as_str().and_then(|s| s.parse::<f64>().ok())
                    .or_else(|| obj["last"].as_f64())
                    .unwrap_or(0.0);

                // KuCoin provides vol (base) and volValue (quote) sometimes
                let base_vol = obj["vol"].as_str().and_then(|s| s.parse::<f64>().ok())
                    .or_else(|| obj["vol"].as_f64())
                    .unwrap_or(0.0);
                let quote_vol = obj["volValue"].as_str().and_then(|s| s.parse::<f64>().ok())
                    .or_else(|| obj["volValue"].as_f64())
                    .unwrap_or(0.0);

                if price > 0.0 {
                    out.push(PairPrice {
                        base: base.to_string(),
                        quote: quote.to_string(),
                        price,
                        is_spot: true,
                        base_volume: base_vol,
                        quote_volume: quote_vol,
                    });
                }
            }
        }
    }

    info!("kucoin returned {} pairs", out.len());
    Ok(out)
}

/// ----------------- BYBIT -----------------
pub async fn fetch_bybit(client: &Client) -> Result<Vec<PairPrice>, String> {
    info!("fetching bybit");

    // Step 1: Get active spot instruments
    let info_url = "https://api.bybit.com/v5/market/instruments-info?category=spot";
    let info: Value = client
        .get(info_url)
        .send()
        .await
        .map_err(|e| format!("bybit instruments error: {}", e))?
        .json()
        .await
        .map_err(|e| format!("bybit decode instruments error: {}", e))?;

    let mut symbol_map: HashMap<String, (String, String)> = HashMap::new();
    if let Some(arr) = info["result"]["list"].as_array() {
        for obj in arr {
            if obj["status"] == "Trading" {
                if let (Some(base), Some(quote), Some(symbol)) = (
                    obj.get("baseCoin"),
                    obj.get("quoteCoin"),
                    obj.get("symbol"),
                ) {
                    let quote = quote.as_str().unwrap().to_uppercase();
                    // keep only common quote markets (same as before)
                    if ["USDT", "USDC", "BTC", "ETH"].contains(&quote.as_str()) {
                        symbol_map.insert(
                            symbol.as_str().unwrap().to_uppercase(),
                            (
                                base.as_str().unwrap().to_uppercase(),
                                quote,
                            ),
                        );
                    }
                }
            }
        }
    }

    // Step 2: fetch live prices + volumes
    let url = "https://api.bybit.com/v5/market/tickers?category=spot";
    let resp: Value = client
        .get(url)
        .send()
        .await
        .map_err(|e| format!("bybit tickers error: {}", e))?
        .json()
        .await
        .map_err(|e| format!("bybit decode tickers error: {}", e))?;

    let mut out = Vec::new();
    if let Some(arr) = resp["result"]["list"].as_array() {
        for obj in arr {
            // symbol, lastPrice, volume (base), quoteVolume24h or turnover24h (quote)
            if let (Some(symbol_v), Some(price_v)) = (obj.get("symbol"), obj.get("lastPrice")) {
                let symbol = symbol_v.as_str().unwrap().to_uppercase();
                if let Some((base, quote)) = symbol_map.get(&symbol) {
                    let price = price_v.as_str().and_then(|s| s.parse::<f64>().ok())
                        .or_else(|| price_v.as_f64())
                        .unwrap_or(0.0);

                    let base_vol = obj.get("volume")
                        .and_then(|v| v.as_str().and_then(|s| s.parse::<f64>().ok()))
                        .or_else(|| obj.get("volume").and_then(|v| v.as_f64()))
                        .unwrap_or(0.0);

                    // try several fields for quote volume
                    let quote_vol = obj.get("quoteVolume24h")
                        .and_then(|v| v.as_str().and_then(|s| s.parse::<f64>().ok()))
                        .or_else(|| obj.get("turnover24h").and_then(|v| v.as_str().and_then(|s| s.parse::<f64>().ok())))
                        .or_else(|| obj.get("turnover24h").and_then(|v| v.as_f64()))
                        .unwrap_or(0.0);

                    if price > 0.0 {
                        out.push(PairPrice {
                            base: base.clone(),
                            quote: quote.clone(),
                            price,
                            is_spot: true,
                            base_volume: base_vol,
                            quote_volume: quote_vol,
                        });
                    }
                }
            }
        }
    }

    info!("bybit returned {} filtered spot pairs", out.len());
    Ok(out)
}

/// ----------------- GATE.IO -----------------
pub async fn fetch_gateio(_client: &Client) -> Result<Vec<PairPrice>, String> {
    info!("fetching gateio");

    // Client that skips SSL cert validation (still useful)
    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .build()
        .map_err(|e| format!("gateio client build error: {}", e))?;

    // Step 1: fetch tradable pairs metadata
    let symbols_url = "https://api.gateio.ws/api/v4/spot/currency_pairs";
    let symbols_resp = client
        .get(symbols_url)
        .send()
        .await
        .map_err(|e| format!("gateio symbols http error: {}", e))?;

    let raw_symbols = symbols_resp
        .text()
        .await
        .map_err(|e| format!("gateio symbols read error: {}", e))?;

    let symbols: Vec<Value> = serde_json::from_str(&raw_symbols)
        .map_err(|e| format!("gateio decode symbols error: {}. First 100 chars: {}", e, &raw_symbols.chars().take(100).collect::<String>()))?;

    let mut tradable = HashSet::new();
    for s in symbols {
        if s["trade_status"] == "tradable" {
            if let Some(id) = s["id"].as_str() {
                tradable.insert(id.to_uppercase());
            }
        }
    }

    // Step 2: fetch tickers
    let url = "https://api.gateio.ws/api/v4/spot/tickers";
    let resp = client
        .get(url)
        .send()
        .await
        .map_err(|e| format!("gateio tickers http error: {}", e))?;

    let raw_tickers = resp
        .text()
        .await
        .map_err(|e| format!("gateio tickers read error: {}", e))?;

    let json: Vec<Value> = serde_json::from_str(&raw_tickers)
        .map_err(|e| format!("gateio decode tickers error: {}. First 100 chars: {}", e, &raw_tickers.chars().take(100).collect::<String>()))?;

    let mut out = Vec::new();
    for v in json {
        let pair = v["currency_pair"].as_str().unwrap_or("");
        if !tradable.contains(&pair.to_uppercase()) {
            continue;
        }
        // price
        let price = v["last"].as_str().and_then(|s| s.parse::<f64>().ok())
            .or_else(|| v["last"].as_f64())
            .unwrap_or(0.0);

        // attempt to get quote and base volumes
        let quote_vol = v["quote_volume"].as_str().and_then(|s| s.parse::<f64>().ok())
            .or_else(|| v["quote_volume"].as_f64())
            .unwrap_or(0.0);
        let base_vol = v["base_volume"].as_str().and_then(|s| s.parse::<f64>().ok())
            .or_else(|| v["base_volume"].as_f64())
            .unwrap_or(0.0);

        if price > 0.0 {
            if let Some((base, quote)) = pair.split_once('_') {
                out.push(PairPrice {
                    base: base.to_string(),
                    quote: quote.to_string(),
                    price,
                    is_spot: true,
                    base_volume: base_vol,
                    quote_volume: quote_vol,
                });
            }
        }
    }

    info!("gateio returned {} spot pairs", out.len());
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
