use reqwest::Client;
use serde_json::Value;
use std::collections::HashMap;

pub async fn fetch_prices(exchange: &str) -> HashMap<String, f64> {
    let client = Client::new();
    let mut prices = HashMap::new();

    match exchange {
        "binance" => {
            if let Ok(resp) = client.get("https://api.binance.com/api/v3/ticker/price").send().await {
                if let Ok(json) = resp.json::<Vec<Value>>().await {
                    for v in json {
                        if let (Some(symbol), Some(price)) = (v["symbol"].as_str(), v["price"].as_str()) {
                            if let Ok(p) = price.parse::<f64>() {
                                prices.insert(symbol.to_uppercase(), p);
                            }
                        }
                    }
                }
            }
        }
        "kucoin" => {
            if let Ok(resp) = client.get("https://api.kucoin.com/api/v1/market/allTickers").send().await {
                if let Ok(json) = resp.json::<Value>().await {
                    if let Some(data) = json["data"]["ticker"].as_array() {
                        for t in data {
                            if let (Some(symbol), Some(price)) = (t["symbol"].as_str(), t["last"].as_str()) {
                                if let Ok(p) = price.parse::<f64>() {
                                    prices.insert(symbol.replace("-", "").to_uppercase(), p);
                                }
                            }
                        }
                    }
                }
            }
        }
        "bybit" => {
            if let Ok(resp) = client.get("https://api.bybit.com/v2/public/tickers").send().await {
                if let Ok(json) = resp.json::<Value>().await {
                    if let Some(data) = json["result"].as_array() {
                        for t in data {
                            if let (Some(symbol), Some(price)) = (t["symbol"].as_str(), t["last_price"].as_str()) {
                                if let Ok(p) = price.parse::<f64>() {
                                    prices.insert(symbol.to_uppercase(), p);
                                }
                            }
                        }
                    }
                }
            }
        }
        "gate" => {
            if let Ok(resp) = client.get("https://api.gate.io/api2/1/tickers").send().await {
                if let Ok(json) = resp.json::<Value>().await {
                    if let Some(obj) = json.as_object() {
                        for (symbol, val) in obj {
                            if let Some(last) = val["last"].as_str() {
                                if let Ok(p) = last.parse::<f64>() {
                                    prices.insert(symbol.replace("_", "").to_uppercase(), p);
                                }
                            }
                        }
                    }
                }
            }
        }
        "kraken" => {
            if let Ok(resp) = client.get("https://api.kraken.com/0/public/Ticker?pair=BTCUSD,ETHUSD").send().await {
                if let Ok(json) = resp.json::<Value>().await {
                    if let Some(obj) = json["result"].as_object() {
                        for (pair, data) in obj {
                            if let Some(price) = data["c"][0].as_str() {
                                if let Ok(p) = price.parse::<f64>() {
                                    prices.insert(pair.to_uppercase(), p);
                                }
                            }
                        }
                    }
                }
            }
        }
        _ => {}
    }

    prices
                                                                  }
