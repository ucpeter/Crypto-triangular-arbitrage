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

        // ---------------- Bybit ----------------
"bybit" => {
    // Get metadata for spot instruments
    let info_url = "https://api.bybit.com/v5/market/instruments-info?category=spot";
    let info: Value = client.get(info_url).send().await?.json().await?;
    let mut symbol_map: HashMap<String, (String, String)> = HashMap::new();

    if let Some(arr) = info["result"]["list"].as_array() {
        for obj in arr {
            // ✅ Only keep pairs that are trading
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

    // Get all prices in one call
    let price_url = "https://api.bybit.com/v5/market/tickers?category=spot";
    let resp: Value = client.get(price_url).send().await?.json().await?;
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
                        }
                    }
                }
            }
        }
    }
                        }

        "gate" | "gateio" => {
    let url = "https://api.gate.io/api/v4/spot/tickers";
    let resp: Value = client.get(url).send().await?.json().await?;
    if let Some(arr) = resp.as_array() {
        for obj in arr {
            if obj["is_disabled"] == false {
                if let (Some(symbol), Some(price_str)) = (obj.get("currency_pair"), obj.get("last")) {
                    let symbol = symbol.as_str().unwrap().to_uppercase();
                    let parts: Vec<&str> = symbol.split('_').collect();
                    if parts.len() == 2 {
                        let base = parts[0].to_string();
                        let quote = parts[1].to_string();
                        if let Ok(price) = price_str.as_str().unwrap().parse::<f64>() {
                            if price > 0.0 {
                                out.push(PairPrice {
                                    base,
                                    quote,
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
        }

        _ => {
            return Err(format!("Exchange {} not supported", exchange).into());
        }
    }

    Ok(out)
                        }
