use std::collections::HashMap;
use reqwest::Error;
use serde::Deserialize;

pub type PriceMap = HashMap<String, f64>;

/// --------------------
/// Binance
/// --------------------
#[derive(Debug, Deserialize)]
struct BinanceTicker {
    symbol: String,
    price: String,
}

pub async fn fetch_binance() -> Result<PriceMap, Error> {
    let url = "https://api.binance.com/api/v3/ticker/price";
    let resp: Vec<BinanceTicker> = reqwest::get(url).await?.json().await?;
    let mut map = PriceMap::new();

    for t in resp {
        if let Ok(price) = t.price.parse::<f64>() {
            // convert "BTCUSDT" -> "BTC/USDT"
            if let Some((base, quote)) = split_symbol(&t.symbol) {
                map.insert(format!("{}/{}", base, quote), price);
            }
        }
    }
    Ok(map)
}

/// --------------------
/// KuCoin
/// --------------------
#[derive(Debug, Deserialize)]
struct KucoinResponse {
    data: Vec<KucoinTicker>,
}

#[derive(Debug, Deserialize)]
struct KucoinTicker {
    symbol: String,
    price: String,
}

pub async fn fetch_kucoin() -> Result<PriceMap, Error> {
    let url = "https://api.kucoin.com/api/v1/market/allTickers";
    let resp: serde_json::Value = reqwest::get(url).await?.json().await?;
    let mut map = PriceMap::new();

    if let Some(arr) = resp["data"]["ticker"].as_array() {
        for t in arr {
            if let (Some(symbol), Some(price_str)) =
                (t["symbol"].as_str(), t["last"].as_str())
            {
                if let Ok(price) = price_str.parse::<f64>() {
                    if let Some((base, quote)) = symbol.split_once('-') {
                        map.insert(format!("{}/{}", base, quote), price);
                    }
                }
            }
        }
    }
    Ok(map)
}

/// --------------------
/// Gate.io
/// --------------------
#[derive(Debug, Deserialize)]
struct GateResponse {
    currency_pair: String,
    last: String,
}

pub async fn fetch_gateio() -> Result<PriceMap, Error> {
    let url = "https://api.gateio.ws/api/v4/spot/tickers";
    let resp: Vec<GateResponse> = reqwest::get(url).await?.json().await?;
    let mut map = PriceMap::new();

    for t in resp {
        if let Ok(price) = t.last.parse::<f64>() {
            if let Some((base, quote)) = t.currency_pair.split_once('_') {
                map.insert(format!("{}/{}", base, quote), price);
            }
        }
    }
    Ok(map)
}

/// --------------------
/// Kraken
/// --------------------
#[derive(Debug, Deserialize)]
struct KrakenResponse {
    result: HashMap<String, serde_json::Value>,
}

pub async fn fetch_kraken() -> Result<PriceMap, Error> {
    let url = "https://api.kraken.com/0/public/Ticker?pair=ALL";
    let resp: KrakenResponse = reqwest::get(url).await?.json().await?;
    let mut map = PriceMap::new();

    for (pair, data) in resp.result {
        if let Some(arr) = data["c"].as_array() {
            if let Some(price_str) = arr[0].as_str() {
                if let Ok(price) = price_str.parse::<f64>() {
                    // Kraken uses weird symbols like "XBTUSDT"
                    if let Some((base, quote)) = normalize_kraken_pair(&pair) {
                        map.insert(format!("{}/{}", base, quote), price);
                    }
                }
            }
        }
    }
    Ok(map)
}

fn normalize_kraken_pair(symbol: &str) -> Option<(String, String)> {
    let mut s = symbol.to_string();
    s = s.replace("XBT", "BTC");
    if s.len() >= 6 {
        let (base, quote) = s.split_at(s.len() - 3);
        Some((base.to_string(), quote.to_string()))
    } else {
        None
    }
}

/// --------------------
/// Bybit (v5 Spot API)
/// --------------------
#[derive(Debug, Deserialize)]
struct BybitResponse {
    retCode: i32,
    result: BybitResult,
}

#[derive(Debug, Deserialize)]
struct BybitResult {
    list: Vec<BybitTicker>,
}

#[derive(Debug, Deserialize)]
struct BybitTicker {
    symbol: String,
    lastPrice: String,
}

pub async fn fetch_bybit() -> Result<PriceMap, Error> {
    let url = "https://api.bybit.com/v5/market/tickers?category=spot";
    let resp: BybitResponse = reqwest::get(url).await?.json().await?;
    let mut map = PriceMap::new();

    for t in resp.result.list {
        if let Ok(price) = t.lastPrice.parse::<f64>() {
            if let Some((base, quote)) = split_symbol(&t.symbol) {
                map.insert(format!("{}/{}", base, quote), price);
            }
        }
    }
    Ok(map)
}

/// --------------------
/// Helpers
/// --------------------
fn split_symbol(symbol: &str) -> Option<(String, String)> {
    // works for Binance-style "BTCUSDT"
    let bases = ["USDT", "BTC", "ETH", "USD", "EUR"];
    for q in bases {
        if symbol.ends_with(q) {
            let base = symbol.trim_end_matches(q).to_string();
            return Some((base, q.to_string()));
        }
    }
    None
    }
