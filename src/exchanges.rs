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

// ---------------- Bybit ----------------
"bybit" => {
    // Step 1: Fetch metadata for spot instruments
    let info_url = "https://api.bybit.com/v5/market/instruments-info?category=spot";
    let info: Value = client.get(info_url).send().await?.json().await?;
    let mut symbol_map: HashMap<String, (String, String)> = HashMap::new();

    if let Some(arr) = info["result"]["list"].as_array() {
        for obj in arr {
            if obj["status"] == "Trading" {
                if let (Some(base), Some(quote), Some(symbol)) =
                    (obj.get("baseCoin"), obj.get("quoteCoin"), obj.get("symbol"))
                {
                    symbol_map.insert(
                        symbol.as_str().unwrap().to_uppercase(),
                        (
                            base.as_str().unwrap().to_uppercase(),
                            quote.as_str().unwrap().to_uppercase(),
                        ),
                    );
                }
            }
        }
    }
    info!("Bybit metadata loaded {} trading spot pairs", symbol_map.len());

    // Step 2: Fetch tickers
    let price_url = "https://api.bybit.com/v5/market/tickers?category=spot";
    let resp: Value = client.get(price_url).send().await?.json().await?;
    let mut kept = 0;
    let mut skipped = 0;

    if let Some(arr) = resp["result"]["list"].as_array() {
        for obj in arr {
            if let (Some(symbol), Some(price_str)) = (obj.get("symbol"), obj.get("lastPrice")) {
                let symbol = symbol.as_str().unwrap().to_uppercase();
                if let Some((base, quote)) = symbol_map.get(&symbol) {
                    if let Ok(price) = price_str.as_str().unwrap().parse::<f64>() {
                        if price > 0.0 {
                            out.push(PairPrice {
                                base: base.clone(),
                                quote: quote.clone(),
                                price,
                                is_spot: true,
                            });
                            kept += 1;
                        }
                    }
                } else {
                    skipped += 1;
                }
            }
        }
    }
    info!("Bybit kept {} pairs, skipped {}", kept, skipped);
}

// ---------------- Gate.io ----------------
"gate" | "gateio" => {
    // Step 1: Get metadata for tradable pairs
    let symbols_url = "https://api.gate.io/api/v4/spot/currency_pairs";
    let symbols_resp = client.get(symbols_url).send().await?.json::<Vec<Value>>().await?;
    let mut tradable: HashMap<String, (String, String)> = HashMap::new();

    for obj in symbols_resp {
        if obj["trade_status"] == "tradable" {
            if let (Some(id), Some(base), Some(quote)) =
                (obj.get("id"), obj.get("base"), obj.get("quote"))
            {
                tradable.insert(
                    id.as_str().unwrap().to_uppercase(),
                    (
                        base.as_str().unwrap().to_uppercase(),
                        quote.as_str().unwrap().to_uppercase(),
                    ),
                );
            }
        }
    }
    info!("Gate.io metadata loaded {} tradable pairs", tradable.len());

    // Step 2: Fetch tickers
    let url = "https://api.gate.io/api/v4/spot/tickers";
    let resp: Value = client.get(url).send().await?.json().await?;
    let mut kept = 0;
    let mut skipped = 0;

    if let Some(arr) = resp.as_array() {
        for obj in arr {
            if let (Some(symbol), Some(price_str)) =
                (obj.get("currency_pair"), obj.get("last"))
            {
                let symbol = symbol.as_str().unwrap().to_uppercase();
                if let Some((base, quote)) = tradable.get(&symbol) {
                    if let Ok(price) = price_str.as_str().unwrap().parse::<f64>() {
                        if price > 0.0 {
                            out.push(PairPrice {
                                base: base.clone(),
                                quote: quote.clone(),
                                price,
                                is_spot: true,
                            });
                            kept += 1;
                        }
                    }
                } else {
                    skipped += 1;
                }
            }
        }
    }
    info!("Gate.io kept {} pairs, skipped {}", kept, skipped);
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
