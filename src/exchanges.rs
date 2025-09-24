use crate::models::PairPrice;
use reqwest::Client;
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use tokio_tungstenite::connect_async;
use futures_util::StreamExt;
use tracing::info;
use tokio::time::{timeout, Duration};

/// ---------------- Binance (Hybrid: WS + REST fallback) ----------------
async fn fetch_binance(client: &Client) -> Result<Vec<PairPrice>, String> {
    info!("fetching binance via websocket (20s) + REST backfill");

    // 1) exchangeInfo for base/quote mapping
    let info_url = "https://api.binance.com/api/v3/exchangeInfo";
    let info_json: Value = client
        .get(info_url)
        .send()
        .await
        .map_err(|e| format!("binance exchangeInfo http error: {}", e))?
        .json()
        .await
        .map_err(|e| format!("binance exchangeInfo decode error: {}", e))?;

    let mut symbol_map: HashMap<String, (String, String)> = HashMap::new();
    if let Some(arr) = info_json["symbols"].as_array() {
        for s in arr {
            if s["status"] == "TRADING" {
                if let (Some(sym), Some(base), Some(quote)) = (
                    s["symbol"].as_str(),
                    s["baseAsset"].as_str(),
                    s["quoteAsset"].as_str(),
                ) {
                    symbol_map.insert(
                        sym.to_uppercase(),
                        (base.to_uppercase(), quote.to_uppercase()),
                    );
                }
            }
        }
    }
    let total_symbols = symbol_map.len();

    // 2) WS snapshot stream (20s)
    let stream_url = "wss://stream.binance.com:9443/ws/!ticker@arr";
    let (ws_stream, _) = connect_async(stream_url)
        .await
        .map_err(|e| format!("binance ws connect error: {}", e))?;
    let (_write, mut read) = ws_stream.split();

    let mut pairs: HashMap<String, PairPrice> = HashMap::new();
    let mut ws_messages = 0usize;

    let ws_result = timeout(Duration::from_secs(20), async {
        while let Some(msg) = read.next().await {
            ws_messages += 1;
            if let Ok(msg) = msg {
                if let Ok(text) = msg.to_text() {
                    if let Ok(arr) = serde_json::from_str::<Value>(text) {
                        if let Some(list) = arr.as_array() {
                            for obj in list {
                                if let (Some(sv), Some(pv), Some(qv)) =
                                    (obj.get("s"), obj.get("c"), obj.get("q"))
                                {
                                    let symbol = sv.as_str().unwrap_or("").to_uppercase();
                                    if let Some((base, quote)) = symbol_map.get(&symbol) {
                                        if let (Ok(price), Ok(vol)) = (
                                            pv.as_str().unwrap_or("0").parse::<f64>(),
                                            qv.as_str().unwrap_or("0").parse::<f64>(),
                                        ) {
                                            if price > 0.0 && vol > 0.0 {
                                                pairs.insert(
                                                    symbol.clone(),
                                                    PairPrice {
                                                        base: base.clone(),
                                                        quote: quote.clone(),
                                                        price,
                                                        is_spot: true,
                                                        liquidity: vol,
                                                    },
                                                );
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    })
    .await;

    if ws_result.is_err() {
        info!("binance ws collection stopped after 20s timeout");
    }

    let ws_pairs = pairs.len();

    // 3) REST fallback for missing pairs
    let rest_url = "https://api.binance.com/api/v3/ticker/24hr";
    let rest_json: Value = client
        .get(rest_url)
        .send()
        .await
        .map_err(|e| format!("binance REST ticker error: {}", e))?
        .json()
        .await
        .map_err(|e| format!("binance REST ticker decode error: {}", e))?;

    let mut rest_added = 0usize;
    if let Some(arr) = rest_json.as_array() {
        for obj in arr {
            if let (Some(sym), Some(price_str), Some(vol_str)) =
                (obj.get("symbol"), obj.get("lastPrice"), obj.get("quoteVolume"))
            {
                let symbol = sym.as_str().unwrap_or("").to_uppercase();
                if !pairs.contains_key(&symbol) {
                    if let Some((base, quote)) = symbol_map.get(&symbol) {
                        if let (Ok(price), Ok(vol)) =
                            (price_str.as_str().unwrap_or("0").parse::<f64>(),
                             vol_str.as_str().unwrap_or("0").parse::<f64>())
                        {
                            if price > 0.0 && vol > 0.0 {
                                pairs.insert(
                                    symbol.clone(),
                                    PairPrice {
                                        base: base.clone(),
                                        quote: quote.clone(),
                                        price,
                                        is_spot: true,
                                        liquidity: vol,
                                    },
                                );
                                rest_added += 1;
                            }
                        }
                    }
                }
            }
        }
    }

    info!(
        "binance: total_symbols={} ws_messages={} ws_pairs={} rest_added={} final_pairs={}",
        total_symbols,
        ws_messages,
        ws_pairs,
        rest_added,
        pairs.len()
    );

    Ok(pairs.into_values().collect())
                }


/// ---------------- KuCoin ----------------
async fn fetch_kucoin(client: &Client) -> Result<Vec<PairPrice>, String> {
    info!("fetching kucoin (REST)");

    // discover tradable symbols
    let sym_url = "https://api.kucoin.com/api/v1/symbols";
    let sym_json: Value = client
        .get(sym_url)
        .send()
        .await
        .map_err(|e| format!("kucoin symbols http error: {}", e))?
        .json()
        .await
        .map_err(|e| format!("kucoin symbols decode error: {}", e))?;

    let mut tradable: HashSet<String> = HashSet::new();
    let mut info_total = 0usize;
    let mut info_skipped = 0usize;
    if let Some(arr) = sym_json["data"].as_array() {
        for s in arr {
            info_total += 1;
            if s["enableTrading"] == true {
                if let Some(sym) = s["symbol"].as_str() {
                    tradable.insert(sym.to_string());
                } else {
                    info_skipped += 1;
                }
            } else {
                info_skipped += 1;
            }
        }
    }

    // fetch tickers
    let url = "https://api.kucoin.com/api/v1/market/allTickers";
    let resp: Value = client
        .get(url)
        .send()
        .await
        .map_err(|e| format!("kucoin tickers http error: {}", e))?
        .json()
        .await
        .map_err(|e| format!("kucoin tickers decode error: {}", e))?;

    let mut out = Vec::new();
    let mut ws_total = 0usize;
    let mut ws_skipped = 0usize;

    if let Some(arr) = resp["data"]["ticker"].as_array() {
        for obj in arr {
            ws_total += 1;
            if let (Some(symbol), Some(price_str), Some(vol_str)) =
                (obj["symbol"].as_str(), obj["last"].as_str(), obj["volValue"].as_str())
            {
                if !tradable.contains(symbol) {
                    ws_skipped += 1;
                    continue;
                }
                if let Some((base, quote)) = symbol.split_once('-') {
                    if let (Ok(price), Ok(vol)) =
                        (price_str.parse::<f64>(), vol_str.parse::<f64>())
                    {
                        if price > 0.0 && vol > 0.0 {
                            out.push(PairPrice {
                                base: base.to_string(),
                                quote: quote.to_string(),
                                price,
                                is_spot: true,
                                liquidity: vol,
                            });
                        } else {
                            ws_skipped += 1;
                        }
                    } else {
                        ws_skipped += 1;
                    }
                } else {
                    ws_skipped += 1;
                }
            } else {
                ws_skipped += 1;
            }
        }
    }

    info!(
        "kucoin: found_total={} info_skipped={} tickers_total={} tickers_skipped={} returned={}",
        info_total,
        info_skipped,
        ws_total,
        ws_skipped,
        out.len()
    );

    Ok(out)
}

/// ----------------- BYBIT -----------------
pub async fn fetch_bybit(client: &Client) -> Result<Vec<PairPrice>, String> {
    info!("fetching bybit (REST)");

    // discovery / mapping
    let info_url = "https://api.bybit.com/v5/market/instruments-info?category=spot";
    let info: Value = client
        .get(info_url)
        .send()
        .await
        .map_err(|e| format!("bybit instruments http error: {}", e))?
        .json()
        .await
        .map_err(|e| format!("bybit instruments decode error: {}", e))?;

    let mut symbol_map: HashMap<String, (String, String)> = HashMap::new();
    let mut info_total = 0usize;
    let mut info_skipped = 0usize;
    if let Some(arr) = info["result"]["list"].as_array() {
        for obj in arr {
            info_total += 1;
            if obj["status"] == "Trading" {
                if let (Some(base), Some(quote), Some(symbol)) = (
                    obj.get("baseCoin"),
                    obj.get("quoteCoin"),
                    obj.get("symbol"),
                ) {
                    let base = base.as_str().unwrap_or("").to_uppercase();
                    let quote = quote.as_str().unwrap_or("").to_uppercase();
                    let symbol = symbol.as_str().unwrap_or("").to_uppercase();

                    if ["USDT", "USDC", "BTC", "ETH"].contains(&quote.as_str()) {
                        symbol_map.insert(symbol, (base, quote));
                    } else {
                        info_skipped += 1;
                    }
                } else {
                    info_skipped += 1;
                }
            } else {
                info_skipped += 1;
            }
        }
    }

    // tickers
    let url = "https://api.bybit.com/v5/market/tickers?category=spot";
    let resp: Value = client
        .get(url)
        .send()
        .await
        .map_err(|e| format!("bybit tickers http error: {}", e))?
        .json()
        .await
        .map_err(|e| format!("bybit tickers decode error: {}", e))?;

    let mut out = Vec::new();
    let mut ws_total = 0usize;
    let mut ws_skipped = 0usize;

    if let Some(arr) = resp["result"]["list"].as_array() {
        // log the first 3 tickers for debugging
        for (i, obj) in arr.iter().enumerate() {
            if i < 3 {
                info!("bybit sample ticker {}: {}", i, obj);
            }
        }

        for obj in arr {
            ws_total += 1;
            if let (Some(symbol_v), Some(price_v)) =
                (obj.get("symbol"), obj.get("lastPrice"))
            {
                let symbol = symbol_v.as_str().unwrap_or("").to_uppercase();

                // pick liquidity field with fallback
                let vol_str = obj.get("quoteVolume24h")
                    .or_else(|| obj.get("turnover24h"))
                    .or_else(|| obj.get("volume24h"));

                if let Some(vol_v) = vol_str {
                    if let Some((base, quote)) = symbol_map.get(&symbol) {
                        let price = price_v.as_str().unwrap_or("0").parse::<f64>().unwrap_or(0.0);
                        let vol = vol_v.as_str().unwrap_or("0").parse::<f64>().unwrap_or(0.0);

                        if price > 0.0 && vol > 0.0 {
                            out.push(PairPrice {
                                base: base.clone(),
                                quote: quote.clone(),
                                price,
                                is_spot: true,
                                liquidity: vol,
                            });
                        } else {
                            ws_skipped += 1;
                        }
                    } else {
                        ws_skipped += 1;
                    }
                } else {
                    ws_skipped += 1;
                }
            } else {
                ws_skipped += 1;
            }
        }
    }

    info!(
        "bybit: found_total={} info_skipped={} tickers_total={} tickers_skipped={} returned={}",
        info_total,
        info_skipped,
        ws_total,
        ws_skipped,
        out.len()
    );

    Ok(out)
                }



/// ----------------- GATE.IO -----------------
pub async fn fetch_gateio(_client: &Client) -> Result<Vec<PairPrice>, String> {
    info!("fetching gateio (REST)");

    let client = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .build()
        .map_err(|e| format!("gateio client build error: {}", e))?;

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
    let symbols: Vec<Value> = serde_json::from_str(&raw_symbols).map_err(|e| {
        format!(
            "gateio decode symbols error: {}. First 100 chars: {}",
            e,
            &raw_symbols.chars().take(100).collect::<String>()
        )
    })?;

    let mut tradable = HashSet::new();
    let mut info_total = 0usize;
    let mut info_skipped = 0usize;
    for s in symbols {
        info_total += 1;
        if s["trade_status"] == "tradable" {
            if let Some(id) = s["id"].as_str() {
                tradable.insert(id.to_uppercase());
            } else {
                info_skipped += 1;
            }
        } else {
            info_skipped += 1;
        }
    }

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
    let json: Vec<Value> = serde_json::from_str(&raw_tickers).map_err(|e| {
        format!(
            "gateio decode tickers error: {}. First 100 chars: {}",
            e,
            &raw_tickers.chars().take(100).collect::<String>()
        )
    })?;

    let mut out = Vec::new();
    let mut ws_total = 0usize;
    let mut ws_skipped = 0usize;
    for v in json {
        ws_total += 1;
        if let (Some(symbol), Some(last_str), Some(vol_str)) =
            (v["currency_pair"].as_str(), v["last"].as_str(), v["quote_volume"].as_str())
        {
            let symbol = symbol.to_uppercase();
            if !tradable.contains(&symbol) {
                ws_skipped += 1;
                continue;
            }
            if let Ok(price) = last_str.parse::<f64>() {
                if price > 0.0 {
                    let parts: Vec<&str> = symbol.split('_').collect();
                    if parts.len() == 2 {
                        if let Ok(vol) = vol_str.parse::<f64>() {
                            if vol > 0.0 {
                                out.push(PairPrice {
                                    base: parts[0].to_string(),
                                    quote: parts[1].to_string(),
                                    price,
                                    is_spot: true,
                                    liquidity: vol,
                                });
                            } else {
                                ws_skipped += 1;
                            }
                        } else {
                            ws_skipped += 1;
                        }
                    } else {
                        ws_skipped += 1;
                    }
                } else {
                    ws_skipped += 1;
                }
            } else {
                ws_skipped += 1;
            }
        } else {
            ws_skipped += 1;
        }
    }

    info!(
        "gateio: found_total={} info_skipped={} tickers_total={} tickers_skipped={} returned={}",
        info_total,
        info_skipped,
        ws_total,
        ws_skipped,
        out.len()
    );

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
