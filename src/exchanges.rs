use reqwest::Client;
use crate::models::Market;
use std::collections::HashMap;

pub async fn fetch_markets(exchange: &str) -> Result<Vec<Market>, Box<dyn std::error::Error>> {
    let url = match exchange {
        "binance" => "https://api.binance.com/api/v3/exchangeInfo",
        "kucoin"  => "https://api.kucoin.com/api/v1/symbols",
        "bybit"   => "https://api.bybit.com/v2/public/symbols",
        "gateio"  => "https://api.gateio.ws/api/v4/spot/currency_pairs",
        _ => return Err("Unsupported exchange".into()),
    };

    let client = Client::new();
    let res = client.get(url).send().await?.json::<serde_json::Value>().await?;
    let mut markets = Vec::new();

    match exchange {
        "binance" => {
            if let Some(arr) = res["symbols"].as_array() {
                for s in arr {
                    markets.push(Market {
                        symbol: s["symbol"].as_str().unwrap_or("").to_string(),
                        base: s["baseAsset"].as_str().unwrap_or("").to_string(),
                        quote: s["quoteAsset"].as_str().unwrap_or("").to_string(),
                        active: s["status"].as_str().unwrap_or("") == "TRADING",
                        spot: true,
                    });
                }
            }
        }
        "kucoin" => {
            if let Some(arr) = res["data"].as_array() {
                for s in arr {
                    markets.push(Market {
                        symbol: s["symbol"].as_str().unwrap_or("").to_string(),
                        base: s["baseCurrency"].as_str().unwrap_or("").to_string(),
                        quote: s["quoteCurrency"].as_str().unwrap_or("").to_string(),
                        active: s["enableTrading"].as_bool().unwrap_or(false),
                        spot: true,
                    });
                }
            }
        }
        "bybit" => {
            if let Some(arr) = res["result"].as_array() {
                for s in arr {
                    markets.push(Market {
                        symbol: s["name"].as_str().unwrap_or("").to_string(),
                        base: s["base_currency"].as_str().unwrap_or("").to_string(),
                        quote: s["quote_currency"].as_str().unwrap_or("").to_string(),
                        active: true,
                        spot: true,
                    });
                }
            }
        }
        "gateio" => {
            if let Some(arr) = res.as_array() {
                for s in arr {
                    markets.push(Market {
                        symbol: s["id"].as_str().unwrap_or("").to_string(),
                        base: s["base"].as_str().unwrap_or("").to_string(),
                        quote: s["quote"].as_str().unwrap_or("").to_string(),
                        active: true,
                        spot: true,
                    });
                }
            }
        }
        _ => {}
    }

    Ok(markets)
}

pub async fn fetch_tickers(exchange: &str) -> Result<HashMap<String, f64>, Box<dyn std::error::Error>> {
    let url = match exchange {
        "binance" => "https://api.binance.com/api/v3/ticker/price",
        "kucoin"  => "https://api.kucoin.com/api/v1/market/allTickers",
        "bybit"   => "https://api.bybit.com/v2/public/tickers",
        "gateio"  => "https://api.gateio.ws/api/v4/spot/tickers",
        _ => return Err("Unsupported exchange".into()),
    };

    let client = Client::new();
    let res = client.get(url).send().await?.json::<serde_json::Value>().await?;
    let mut tickers = HashMap::new();

    match exchange {
        "binance" => {
            if let Some(arr) = res.as_array() {
                for t in arr {
                    if let (Some(sym), Some(price)) = (t["symbol"].as_str(), t["price"].as_str()) {
                        tickers.insert(sym.to_string(), price.parse::<f64>().unwrap_or(0.0));
                    }
                }
            }
        }
        "kucoin" => {
            if let Some(arr) = res["data"]["ticker"].as_array() {
                for t in arr {
                    if let (Some(sym), Some(price)) = (t["symbol"].as_str(), t["last"].as_str()) {
                        tickers.insert(sym.replace("-", ""), price.parse::<f64>().unwrap_or(0.0));
                    }
                }
            }
        }
        "bybit" => {
            if let Some(arr) = res["result"].as_array() {
                for t in arr {
                    if let (Some(sym), Some(price)) = (t["symbol"].as_str(), t["last_price"].as_str()) {
                        tickers.insert(sym.to_string(), price.parse::<f64>().unwrap_or(0.0));
                    }
                }
            }
        }
        "gateio" => {
            if let Some(arr) = res.as_array() {
                for t in arr {
                    if let (Some(sym), Some(price)) = (t["currency_pair"].as_str(), t["last"].as_str()) {
                        tickers.insert(sym.replace("_", "").to_uppercase(), price.parse::<f64>().unwrap_or(0.0));
                    }
                }
            }
        }
        _ => {}
    }

    Ok(tickers)
}
