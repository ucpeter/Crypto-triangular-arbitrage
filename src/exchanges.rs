use crate::models::PairPrice;
use reqwest::Client;
use serde_json::Value;
use std::error::Error;
use std::collections::HashMap;

/// Fetch all spot pairs for a given exchange
pub async fn fetch_exchange_data(exchange: &str) -> Result<Vec<PairPrice>, Box<dyn Error>> {
    let client = Client::new();
    let mut out: Vec<PairPrice> = Vec::new();

    match exchange {
        // ---------------- Binance ----------------
"binance" => {
    // Use exchangeInfo instead of ticker/price for proper base/quote
    let url = "https://api.binance.com/api/v3/exchangeInfo";
    let resp: Value = client.get(url).send().await?.json().await?;
    if let Some(arr) = resp["symbols"].as_array() {
        for obj in arr {
            if obj["status"] == "TRADING" && obj["isSpotTradingAllowed"] == true {
                if let (Some(base), Some(quote)) =
                    (obj.get("baseAsset"), obj.get("quoteAsset"))
                {
                    let base = base.as_str().unwrap().to_uppercase();
                    let quote = quote.as_str().unwrap().to_uppercase();
                    // Now get price from ticker/price
                    let sym = obj["symbol"].as_str().unwrap();
                    let price_url = format!("https://api.binance.com/api/v3/ticker/price?symbol={}", sym);
                    if let Ok(p) = client.get(&price_url).send().await?.json::<Value>().await {
                        if let Some(price_str) = p["price"].as_str() {
                            if let Ok(price) = price_str.parse::<f64>() {
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
}

// ---------------- Bybit ----------------
"bybit" => {
    // Instruments-info gives proper spot pairs
    let url = "https://api.bybit.com/v5/market/instruments-info?category=spot";
    let resp: Value = client.get(url).send().await?.json().await?;
    if let Some(arr) = resp["result"]["list"].as_array() {
        for obj in arr {
            if let (Some(base), Some(quote), Some(sym)) =
                (obj.get("baseCoin"), obj.get("quoteCoin"), obj.get("symbol"))
            {
                let base = base.as_str().unwrap().to_uppercase();
                let quote = quote.as_str().unwrap().to_uppercase();
                let symbol = sym.as_str().unwrap();
                let price_url = format!("https://api.bybit.com/v5/market/tickers?category=spot&symbol={}", symbol);
                if let Ok(p) = client.get(&price_url).send().await?.json::<Value>().await {
                    if let Some(list) = p["result"]["list"].as_array() {
                        if let Some(first) = list.first() {
                            if let Some(price_str) = first["lastPrice"].as_str() {
                                if let Ok(price) = price_str.parse::<f64>() {
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
    }
        }
        

        // ---------------- KuCoin ----------------
        "kucoin" => {
            let url = "https://api.kucoin.com/api/v1/market/allTickers";
            let resp: Value = client.get(url).send().await?.json().await?;
            if let Some(arr) = resp["data"]["ticker"].as_array() {
                for obj in arr {
                    if let (Some(symbol), Some(price_str)) =
                        (obj.get("symbol"), obj.get("last"))
                    {
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

        

        // ---------------- Gate.io ----------------
        "gate" | "gateio" => {
            let url = "https://api.gate.io/api/v4/spot/tickers";
            let resp: Value = client.get(url).send().await?.json().await?;
            if let Some(arr) = resp.as_array() {
                for obj in arr {
                    if let (Some(symbol), Some(price_str)) =
                        (obj.get("currency_pair"), obj.get("last"))
                    {
                        let symbol = symbol.as_str().unwrap().to_uppercase();
                        let price: f64 = price_str.as_str().unwrap().parse().unwrap_or(0.0);

                        if let Some((base, quote)) = symbol.split_once("_") {
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

        _ => {
            return Err(format!("Exchange {} not supported", exchange).into());
        }
    }

    Ok(out)
}

/// Helper: split symbol like "ETHUSDT" into (ETH, USDT)
fn split_symbol(symbol: &str) -> Option<(String, String)> {
    let known_quotes = ["USDT", "USDC", "BTC", "ETH", "BNB"];
    for q in &known_quotes {
        if symbol.ends_with(q) {
            let len = symbol.len() - q.len();
            return Some((symbol[..len].to_string(), q.to_string()));
        }
    }
    None
                        }
