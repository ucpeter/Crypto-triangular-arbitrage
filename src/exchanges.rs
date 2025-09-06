use crate::models::PairPrice;
use reqwest::Client;
use serde_json::Value;
use std::collections::HashMap;
use std::error::Error;

/// Fetch all spot pairs for a given exchange
pub async fn fetch_exchange_data(exchange: &str) -> Result<Vec<PairPrice>, Box<dyn Error>> {
    let client = Client::new();
    let mut out: Vec<PairPrice> = Vec::new();

    match exchange {
        // ---------------- Binance ----------------
        "binance" => {
            // Get symbol metadata (base/quote mapping)
            let info_url = "https://api.binance.com/api/v3/exchangeInfo";
            let info: Value = client.get(info_url).send().await?.json().await?;
            let mut symbol_map: HashMap<String, (String, String)> = HashMap::new();

            if let Some(arr) = info["symbols"].as_array() {
                for obj in arr {
                    if obj["status"] == "TRADING" && obj["isSpotTradingAllowed"] == true {
                        if let (Some(base), Some(quote), Some(symbol)) =
                            (obj.get("baseAsset"), obj.get("quoteAsset"), obj.get("symbol"))
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

            // Get all live prices in one call
            let price_url = "https://api.binance.com/api/v3/ticker/price";
            let prices: Value = client.get(price_url).send().await?.json().await?;
            if let Some(arr) = prices.as_array() {
                for obj in arr {
                    if let (Some(symbol), Some(price_str)) = (obj.get("symbol"), obj.get("price")) {
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
                                }
                            }
                        }
                    }
                }
            }
        }

        // ---------------- KuCoin ----------------
        "kucoin" => {
            let url = "https://api.kucoin.com/api/v1/market/allTickers";
            let resp: Value = client.get(url).send().await?.json().await?;
            if let Some(arr) = resp["data"]["ticker"].as_array() {
                for obj in arr {
                    if let (Some(symbol), Some(price_str)) = (obj.get("symbol"), obj.get("last")) {
                        let symbol = symbol.as_str().unwrap().to_uppercase();
                        let price: f64 = price_str.as_str().unwrap().parse().unwrap_or(0.0);

                        if let Some((base, quote)) = symbol.split_once("-") {
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

        /// ----------------- BYBIT -----------------
pub async fn fetch_bybit(client: &Client) -> Result<Vec<PairPrice>, String> {
    info!("fetching bybit");

    // Step 1: fetch active spot symbols
    let symbols_url = "https://api.bybit.com/v5/market/instruments-info?category=spot";
    let symbols_resp = client.get(symbols_url)
        .send().await
        .map_err(|e| format!("bybit http error: {}", e))?;
    let symbols_json: serde_json::Value = symbols_resp.json().await
        .map_err(|e| format!("bybit decode error: {}", e))?;

    let mut active: HashSet<String> = HashSet::new();
    if let Some(list) = symbols_json["result"]["list"].as_array() {
        for v in list {
            if v["status"].as_str() == Some("Trading") {
                if let Some(s) = v["symbol"].as_str() {
                    active.insert(s.to_string());
                }
            }
        }
    }

    // Step 2: fetch live tickers
    let tickers_url = "https://api.bybit.com/v5/market/tickers?category=spot";
    let resp = client.get(tickers_url)
        .send().await
        .map_err(|e| format!("bybit ticker http error: {}", e))?;
    let json: serde_json::Value = resp.json().await
        .map_err(|e| format!("bybit ticker decode error: {}", e))?;

    let mut out = Vec::new();
    if let Some(arr) = json["result"]["list"].as_array() {
        for v in arr {
            if let (Some(symbol), Some(price_str)) = (v["symbol"].as_str(), v["lastPrice"].as_str()) {
                if !active.contains(symbol) {
                    continue; // skip delisted
                }
                if let Ok(price) = price_str.parse::<f64>() {
                    let (base, quote) = split_symbol(symbol);
                    if !base.is_empty() && !quote.is_empty() {
                        out.push(PairPrice { base, quote, price, is_spot: true });
                    }
                }
            }
        }
    }

    info!("bybit returned {} spot pairs", out.len());
    Ok(out)
}

// ----------------- GATE.IO -----------------
pub async fn fetch_gateio(client: &Client) -> Result<Vec<PairPrice>, String> {
    info!("fetching gateio");

    // Step 1: fetch tradable pairs
    let symbols_url = "https://api.gate.io/api/v4/spot/currency_pairs";
    let symbols_resp = client.get(symbols_url)
        .send().await
        .map_err(|e| format!("gateio http error: {}", e))?;
    let symbols: Vec<serde_json::Value> = symbols_resp.json().await
        .map_err(|e| format!("gateio decode error: {}", e))?;

    let mut tradable: HashSet<String> = HashSet::new();
    for s in symbols {
        if s["trade_status"] == "tradable" {
            if let Some(id) = s["id"].as_str() {
                tradable.insert(id.to_string());
            }
        }
    }

    // Step 2: fetch tickers
    let tickers_url = "https://api.gate.io/api/v4/spot/tickers";
    let resp = client.get(tickers_url)
        .send().await
        .map_err(|e| format!("gateio ticker http error: {}", e))?;
    let json: Vec<serde_json::Value> = resp.json().await
        .map_err(|e| format!("gateio ticker decode error: {}", e))?;

    let mut out = Vec::new();
    for v in json {
        if let (Some(symbol), Some(price_str)) = (v["currency_pair"].as_str(), v["last"].as_str()) {
            if !tradable.contains(symbol) {
                continue;
            }
            if let Ok(price) = price_str.parse::<f64>() {
                let parts: Vec<&str> = symbol.split('_').collect();
                if parts.len() == 2 {
                    out.push(PairPrice {
                        base: parts[0].to_string(),
                        quote: parts[1].to_string(),
                        price,
                        is_spot: true,
                    });
                }
            }
        }
    }

    info!("gateio returned {} spot pairs", out.len());
    Ok(out)
} // ✅ this closes fetch_gateio properly

// ----------------- FETCH DISPATCH -----------------
pub async fn fetch_exchange_data(exchange: &str) -> Result<Vec<PairPrice>, Box<dyn std::error::Error>> {
    let client = Client::new();
    match exchange {
        "binance" => Ok(fetch_binance(&client).await?),
        "bybit"   => Ok(fetch_bybit(&client).await?),
        "kucoin"  => Ok(fetch_kucoin(&client).await?),
        "gateio"  => Ok(fetch_gateio(&client).await?),
        _ => Err(format!("unsupported exchange: {}", exchange).into()),
    }
} // ✅ this closes fetch_exchange_data properly
