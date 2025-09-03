use crate::models::PriceMap;
use reqwest::Client;
use std::collections::HashMap;

pub async fn fetch_exchange_data(exchange: &str) -> Result<PriceMap, String> {
    let client = Client::new();
    match exchange.to_lowercase().as_str() {
        "binance" => fetch_binance(&client).await,
        "kucoin" => fetch_kucoin(&client).await,
        "kraken" => fetch_kraken(&client).await,
        "gateio" => fetch_gateio(&client).await,
        "bybit" => fetch_bybit(&client).await,
        _ => Err(format!("Exchange {} not supported", exchange)),
    }
}

/// Binance Spot API
async fn fetch_binance(client: &Client) -> Result<PriceMap, String> {
    let url = "https://api.binance.com/api/v3/ticker/price";
    let res = client.get(url).send().await.map_err(|e| e.to_string())?;
    let data: Vec<HashMap<String, String>> = res.json().await.map_err(|e| e.to_string())?;

    let mut map = PriceMap::new();
    for item in data {
        if let (Some(symbol), Some(price_str)) = (item.get("symbol"), item.get("price")) {
            if let Ok(price) = price_str.parse::<f64>() {
                // Convert SYMBOL (e.g. ETHBTC) into ETH/BTC format
                if symbol.len() > 3 {
                    let (base, quote) = symbol.split_at(symbol.len() - 3);
                    map.insert(format!("{}/{}", base, quote), price);
                }
            }
        }
    }
    Ok(map)
}

/// KuCoin Spot API
async fn fetch_kucoin(client: &Client) -> Result<PriceMap, String> {
    let url = "https://api.kucoin.com/api/v1/market/allTickers";
    let res = client.get(url).send().await.map_err(|e| e.to_string())?;
    let data: serde_json::Value = res.json().await.map_err(|e| e.to_string())?;

    let mut map = PriceMap::new();
    if let Some(tickers) = data["data"]["ticker"].as_array() {
        for t in tickers {
            if let (Some(symbol), Some(price)) = (t["symbol"].as_str(), t["last"].as_str()) {
                if let Ok(p) = price.parse::<f64>() {
                    // KuCoin uses format like "ETH-BTC"
                    map.insert(symbol.replace("-", "/"), p);
                }
            }
        }
    }
    Ok(map)
}

/// Kraken Spot API
async fn fetch_kraken(client: &Client) -> Result<PriceMap, String> {
    let url = "https://api.kraken.com/0/public/Ticker?pair=BTCUSD,ETHUSD,ETHBTC";
    let res = client.get(url).send().await.map_err(|e| e.to_string())?;
    let data: serde_json::Value = res.json().await.map_err(|e| e.to_string())?;

    let mut map = PriceMap::new();
    if let Some(result) = data["result"].as_object() {
        for (pair, info) in result {
            if let Some(price_str) = info["c"][0].as_str() {
                if let Ok(price) = price_str.parse::<f64>() {
                    // Kraken pairs sometimes look like "XETHZUSD"
                    let pair_fmt = normalize_kraken_pair(pair);
                    map.insert(pair_fmt, price);
                }
            }
        }
    }
    Ok(map)
}

fn normalize_kraken_pair(raw: &str) -> String {
    // crude normalization
    let mut s = raw.to_string();
    s = s.replace("XBT", "BTC");
    if s.len() > 3 {
        let (base, quote) = s.split_at(s.len() - 3);
        return format!("{}/{}", base, quote);
    }
    s
}

/// Gate.io Spot API
async fn fetch_gateio(client: &Client) -> Result<PriceMap, String> {
    let url = "https://api.gateio.ws/api/v4/spot/tickers";
    let res = client.get(url).send().await.map_err(|e| e.to_string())?;
    let data: Vec<serde_json::Value> = res.json().await.map_err(|e| e.to_string())?;

    let mut map = PriceMap::new();
    for t in data {
        if let (Some(symbol), Some(price_str)) = (t["currency_pair"].as_str(), t["last"].as_str()) {
            if let Ok(p) = price_str.parse::<f64>() {
                map.insert(symbol.replace("_", "/"), p);
            }
        }
    }
    Ok(map)
}

/// Bybit Spot API
async fn fetch_bybit(client: &Client) -> Result<PriceMap, String> {
    let url = "https://api.bybit.com/v2/public/tickers";
    let res = client.get(url).send().await.map_err(|e| e.to_string())?;
    let data: serde_json::Value = res.json().await.map_err(|e| e.to_string())?;

    let mut map = PriceMap::new();
    if let Some(tickers) = data["result"].as_array() {
        for t in tickers {
            if let (Some(symbol), Some(price_str)) = (t["symbol"].as_str(), t["last_price"].as_str())
            {
                if let Ok(p) = price_str.parse::<f64>() {
                    // Bybit uses symbols like ETHUSDT → convert to ETH/USDT
                    if symbol.len() > 3 {
                        let (base, quote) = symbol.split_at(symbol.len() - 4);
                        map.insert(format!("{}/{}", base, quote), p);
                    }
                }
            }
        }
    }
    Ok(map)
    }
