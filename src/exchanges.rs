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
    let info_url = "https://api.bybit.com/v5/market/instruments-info?category=spot";
    let info: serde_json::Value = client
        .get(info_url)
        .send()
        .await
        .map_err(|e| format!("bybit info http error: {}", e))?
        .json()
        .await
        .map_err(|e| format!("bybit info decode error: {}", e))?;

    let mut symbol_map: std::collections::HashMap<String, (String, String)> = std::collections::HashMap::new();

    if let Some(arr) = info["result"]["list"].as_array() {
        for obj in arr {
            if obj["status"] == "Trading" {
                if let (Some(base), Some(quote), Some(symbol)) =
                    (obj["baseCoin"].as_str(), obj["quoteCoin"].as_str(), obj["symbol"].as_str())
                {
                    symbol_map.insert(
                        symbol.to_uppercase(),
                        (base.to_uppercase(), quote.to_uppercase()),
                    );
                }
            }
        }
    }

    let price_url = "https://api.bybit.com/v5/market/tickers?category=spot";
    let resp: serde_json::Value = client
        .get(price_url)
        .send()
        .await
        .map_err(|e| format!("bybit price http error: {}", e))?
        .json()
        .await
        .map_err(|e| format!("bybit price decode error: {}", e))?;

    let mut out = Vec::new();
    if let Some(arr) = resp["result"]["list"].as_array() {
        for obj in arr {
            if let (Some(symbol), Some(price_str)) = (obj["symbol"].as_str(), obj["lastPrice"].as_str()) {
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

    info!("bybit returned {} pairs", out.len());
    Ok(out)
            }

/// ----------------- GATE.IO -----------------
pub async fn fetch_gateio(client: &Client) -> Result<Vec<PairPrice>, String> {
    let url = "https://api.gate.io/api/v4/spot/currency_pairs";
    let symbols: Vec<serde_json::Value> = client
        .get(url)
        .send()
        .await
        .map_err(|e| format!("gateio symbols http error: {}", e))?
        .json()
        .await
        .map_err(|e| format!("gateio symbols decode error: {}", e))?;

    let mut tradable = std::collections::HashSet::new();
    for s in symbols {
        if s["trade_status"] == "tradable" {
            if let Some(sym) = s["id"].as_str() {
                tradable.insert(sym.to_uppercase());
            }
        }
    }

    let tickers_url = "https://api.gate.io/api/v4/spot/tickers";
    let tickers: Vec<serde_json::Value> = client
        .get(tickers_url)
        .send()
        .await
        .map_err(|e| format!("gateio tickers http error: {}", e))?
        .json()
        .await
        .map_err(|e| format!("gateio tickers decode error: {}", e))?;

    let mut out = Vec::new();
    for t in tickers {
        if let (Some(symbol), Some(price_str)) =
            (t["currency_pair"].as_str(), t["last"].as_str())
        {
            let symbol = symbol.to_uppercase();
            if !tradable.contains(&symbol) {
                continue;
            }
            let parts: Vec<&str> = symbol.split('_').collect();
            if parts.len() == 2 {
                if let Ok(price) = price_str.parse::<f64>() {
                    if price > 0.0 {
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
    }

    info!("gateio returned {} pairs", out.len());
    Ok(out)
}

pub async fn fetch_exchange_data(exchange: &str) -> Result<Vec<PairPrice>, String> {
    let client = Client::new();

    match exchange {
        "binance" => fetch_binance(&client).await,
        "kucoin" => fetch_kucoin(&client).await,
        "gate" | "gateio" => fetch_gateio(&client).await,
        "bybit" => fetch_bybit(&client).await,
        _ => Err(format!("unsupported exchange: {}", exchange)),
    }
}
