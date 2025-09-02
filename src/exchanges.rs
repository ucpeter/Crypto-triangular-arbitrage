use std::collections::HashMap;
use reqwest::Client;
use serde_json::Value;
use tokio::time::{sleep, Duration};

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

/// Fetch spot prices from Binance
pub async fn fetch_binance(client: &Client) -> PriceMap {
    let mut prices = PriceMap::new();
    let url = "https://api.binance.com/api/v3/ticker/price";
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
            Err(e) => eprintln!("Binance JSON parse error: {:?}", e),
        },
        Err(e) => eprintln!("Binance fetch error: {:?}", e),
    }
    println!("Binance loaded {} pairs", prices.len());
    prices
}

/// Fetch spot prices from KuCoin
pub async fn fetch_kucoin(client: &Client) -> PriceMap {
    let mut prices = PriceMap::new();
    let url = "https://api.kucoin.com/api/v1/market/allTickers";
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
            Err(e) => eprintln!("KuCoin JSON parse error: {:?}", e),
        },
        Err(e) => eprintln!("KuCoin fetch error: {:?}", e),
    }
    println!("KuCoin loaded {} pairs", prices.len());
    prices
}

/// Fetch spot prices from Kraken
pub async fn fetch_kraken(client: &Client) -> PriceMap {
    let mut prices = PriceMap::new();
    let url = "https://api.kraken.com/0/public/Ticker?pair=ALL";
    match client.get(url).send().await {
        Ok(resp) => match resp.json::<Value>().await {
            Ok(json) => {
                if let Some(result) = json["result"].as_object() {
                    for (symbol, data) in result {
                        if let Some(c_array) = data["c"].as_array() {
                            if let Some(price_str) = c_array.get(0).and_then(|v| v.as_str()) {
                                let normalized = symbol.replace("XBT", "BTC").replace("XDG", "DOGE");
                                let base = &normalized[..3];
                                let quote = &normalized[3..];
                                let pair = format!("{}/{}", base, quote);
                                if let Ok(price) = price_str.parse::<f64>() {
                                    prices.insert(pair, price);
                                }
                            }
                        }
                    }
                }
            }
            Err(e) => eprintln!("Kraken JSON parse error: {:?}", e),
        },
        Err(e) => eprintln!("Kraken fetch error: {:?}", e),
    }
    println!("Kraken loaded {} pairs", prices.len());
    prices
}

/// Fetch spot prices from Gate.io
pub async fn fetch_gateio(client: &Client) -> PriceMap {
    let mut prices = PriceMap::new();
    let url = "https://api.gateio.ws/api/v4/spot/tickers";
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
            Err(e) => eprintln!("Gate.io JSON parse error: {:?}", e),
        },
        Err(e) => eprintln!("Gate.io fetch error: {:?}", e),
    }
    println!("Gate.io loaded {} pairs", prices.len());
    prices
}

/// Fetch spot prices from Bybit
pub async fn fetch_bybit(client: &Client) -> PriceMap {
    let mut prices = PriceMap::new();
    let url = "https://api.bybit.com/v5/market/tickers?category=spot";
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
            Err(e) => eprintln!("Bybit JSON parse error: {:?}", e),
        },
        Err(e) => eprintln!("Bybit fetch error: {:?}", e),
    }
    println!("Bybit loaded {} pairs", prices.len());
    prices
}

/// Main fetcher to gather data for all exchanges
pub async fn fetch_all_exchanges() -> HashMap<String, PriceMap> {
    let client = Client::new();

    let (binance, kucoin, kraken, gateio, bybit) = tokio::join!(
        fetch_binance(&client),
        fetch_kucoin(&client),
        fetch_kraken(&client),
        fetch_gateio(&client),
        fetch_bybit(&client)
    );

    let mut result = HashMap::new();
    result.insert("Binance".to_string(), binance);
    result.insert("KuCoin".to_string(), kucoin);
    result.insert("Kraken".to_string(), kraken);
    result.insert("Gate.io".to_string(), gateio);
    result.insert("Bybit".to_string(), bybit);

    result
                            }
