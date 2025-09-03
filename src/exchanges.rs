use std::collections::HashMap;
use reqwest::Client;
use serde_json::Value;

pub type PriceMap = HashMap<String, f64>;

/// Normalize symbols like BTCUSDT → BTC/USDT
fn normalize_symbol(symbol: &str) -> Option<String> {
    let known_quotes = ["USDT", "USDC", "BTC", "ETH", "BUSD", "EUR"];
    for quote in known_quotes {
        if symbol.ends_with(quote) && symbol.len() > quote.len() {
            let base = &symbol[..symbol.len() - quote.len()];
            return Some(format!("{}/{}", base, quote));
        }
    }
    None
}

/// Binance spot prices
pub async fn fetch_binance() -> Result<PriceMap, String> {
    let client = Client::new();
    let url = "https://api.binance.com/api/v3/ticker/price";
    let mut prices = PriceMap::new();

    match client.get(url).send().await {
        Ok(resp) => match resp.json::<Value>().await {
            Ok(json) => {
                if let Some(arr) = json.as_array() {
                    for entry in arr {
                        if let (Some(symbol), Some(price_str)) =
                            (entry["symbol"].as_str(), entry["price"].as_str())
                        {
                            if let Some(pair) = normalize_symbol(symbol) {
                                if let Ok(price) = price_str.parse::<f64>() {
                                    prices.insert(pair, price);
                                }
                            }
                        }
                    }
                }
            }
            Err(e) => return Err(format!("Binance JSON parse error: {:?}", e)),
        },
        Err(e) => return Err(format!("Binance fetch error: {:?}", e)),
    }

    println!("Binance loaded {} pairs", prices.len());
    Ok(prices)
}

/// KuCoin spot prices
pub async fn fetch_kucoin() -> Result<PriceMap, String> {
    let client = Client::new();
    let url = "https://api.kucoin.com/api/v1/market/allTickers";
    let mut prices = PriceMap::new();

    match client.get(url).send().await {
        Ok(resp) => match resp.json::<Value>().await {
            Ok(json) => {
                if let Some(arr) = json["data"]["ticker"].as_array() {
                    for entry in arr {
                        if let (Some(symbol), Some(price_str)) =
                            (entry["symbol"].as_str(), entry["last"].as_str())
                        {
                            let pair = symbol.replace("-", "/");
                            if let Ok(price) = price_str.parse::<f64>() {
                                prices.insert(pair, price);
                            }
                        }
                    }
                }
            }
            Err(e) => return Err(format!("KuCoin JSON parse error: {:?}", e)),
        },
        Err(e) => return Err(format!("KuCoin fetch error: {:?}", e)),
    }

    println!("KuCoin loaded {} pairs", prices.len());
    Ok(prices)
}

/// Kraken spot prices
pub async fn fetch_kraken() -> Result<PriceMap, String> {
    let client = Client::new();
    let url = "https://api.kraken.com/0/public/Ticker?pair=ALL";
    let mut prices = PriceMap::new();

    match client.get(url).send().await {
        Ok(resp) => match resp.json::<Value>().await {
            Ok(json) => {
                if let Some(result) = json["result"].as_object() {
                    for (symbol, data) in result {
                        if let Some(c_array) = data["c"].as_array() {
                            if let Some(price_str) = c_array.get(0).and_then(|v| v.as_str()) {
                                let normalized = symbol.replace("XBT", "BTC").replace("XDG", "DOGE");
                                if normalized.len() > 3 {
                                    let (base, quote) = normalized.split_at(3);
                                    let pair = format!("{}/{}", base, quote);
                                    if let Ok(price) = price_str.parse::<f64>() {
                                        prices.insert(pair, price);
                                    }
                                }
                            }
                        }
                    }
                }
            }
            Err(e) => return Err(format!("Kraken JSON parse error: {:?}", e)),
        },
        Err(e) => return Err(format!("Kraken fetch error: {:?}", e)),
    }

    println!("Kraken loaded {} pairs", prices.len());
    Ok(prices)
}

/// Gate.io spot prices
pub async fn fetch_gateio() -> Result<PriceMap, String> {
    let client = Client::new();
    let url = "https://api.gateio.ws/api/v4/spot/tickers";
    let mut prices = PriceMap::new();

    match client.get(url).send().await {
        Ok(resp) => match resp.json::<Value>().await {
            Ok(json) => {
                if let Some(arr) = json.as_array() {
                    for entry in arr {
                        if let (Some(symbol), Some(price_str)) =
                            (entry["currency_pair"].as_str(), entry["last"].as_str())
                        {
                            let pair = symbol.replace("_", "/");
                            if let Ok(price) = price_str.parse::<f64>() {
                                prices.insert(pair, price);
                            }
                        }
                    }
                }
            }
            Err(e) => return Err(format!("Gate.io JSON parse error: {:?}", e)),
        },
        Err(e) => return Err(format!("Gate.io fetch error: {:?}", e)),
    }

    println!("Gate.io loaded {} pairs", prices.len());
    Ok(prices)
}

/// Bybit spot prices
pub async fn fetch_bybit() -> Result<PriceMap, String> {
    let client = Client::new();
    let url = "https://api.bybit.com/v5/market/tickers?category=spot";
    let mut prices = PriceMap::new();

    match client.get(url).send().await {
        Ok(resp) => match resp.json::<Value>().await {
            Ok(json) => {
                if let Some(arr) = json["result"]["list"].as_array() {
                    for entry in arr {
                        if let (Some(symbol), Some(price_str)) =
                            (entry["symbol"].as_str(), entry["lastPrice"].as_str())
                        {
                            if let Some(pair) = normalize_symbol(symbol) {
                                if let Ok(price) = price_str.parse::<f64>() {
                                    prices.insert(pair, price);
                                }
                            }
                        }
                    }
                }
            }
            Err(e) => return Err(format!("Bybit JSON parse error: {:?}", e)),
        },
        Err(e) => return Err(format!("Bybit fetch error: {:?}", e)),
    }

    println!("Bybit loaded {} pairs", prices.len());
    Ok(prices)
                    }
