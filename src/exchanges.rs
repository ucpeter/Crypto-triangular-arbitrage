use crate::models::PairPrice;
use reqwest::Error;
use serde::Deserialize;

/// --------------------
/// Binance
/// --------------------
#[derive(Debug, Deserialize)]
struct BinanceTicker {
    symbol: String,
    price: String,
}

pub async fn fetch_binance() -> Result<Vec<PairPrice>, Error> {
    let url = "https://api.binance.com/api/v3/ticker/price";
    let resp: Vec<BinanceTicker> = reqwest::get(url).await?.json().await?;
    let mut pairs = Vec::new();

    for t in resp {
        if let Ok(price) = t.price.parse::<f64>() {
            if let Some((base, quote)) = split_symbol(&t.symbol) {
                pairs.push(PairPrice {
                    base,
                    quote,
                    price,
                    is_spot: true,
                });
            }
        }
    }
    Ok(pairs)
}

/// --------------------
/// KuCoin
/// --------------------
#[derive(Debug, Deserialize)]
struct KucoinResponse {
    data: KucoinData,
}

#[derive(Debug, Deserialize)]
struct KucoinData {
    ticker: Vec<KucoinTicker>,
}

#[derive(Debug, Deserialize)]
struct KucoinTicker {
    symbol: String,
    last: String,
}

pub async fn fetch_kucoin() -> Result<Vec<PairPrice>, Error> {
    let url = "https://api.kucoin.com/api/v1/market/allTickers";
    let resp: KucoinResponse = reqwest::get(url).await?.json().await?;
    let mut pairs = Vec::new();

    for t in resp.data.ticker {
        if let Ok(price) = t.last.parse::<f64>() {
            if let Some((base, quote)) = t.symbol.split_once('-') {
                pairs.push(PairPrice {
                    base: base.to_string(),
                    quote: quote.to_string(),
                    price,
                    is_spot: true,
                });
            }
        }
    }
    Ok(pairs)
}

/// --------------------
/// Gate.io
/// --------------------
#[derive(Debug, Deserialize)]
struct GateResponse {
    currency_pair: String,
    last: String,
}

pub async fn fetch_gateio() -> Result<Vec<PairPrice>, Error> {
    let url = "https://api.gateio.ws/api/v4/spot/tickers";
    let resp: Vec<GateResponse> = reqwest::get(url).await?.json().await?;
    let mut pairs = Vec::new();

    for t in resp {
        if let Ok(price) = t.last.parse::<f64>() {
            if let Some((base, quote)) = t.currency_pair.split_once('_') {
                pairs.push(PairPrice {
                    base: base.to_string(),
                    quote: quote.to_string(),
                    price,
                    is_spot: true,
                });
            }
        }
    }
    Ok(pairs)
}

/// --------------------
/// Kraken
/// --------------------
#[derive(Debug, Deserialize)]
struct KrakenResponse {
    result: std::collections::HashMap<String, serde_json::Value>,
}

pub async fn fetch_kraken() -> Result<Vec<PairPrice>, Error> {
    let url = "https://api.kraken.com/0/public/Ticker?pair=ALL";
    let resp: KrakenResponse = reqwest::get(url).await?.json().await?;
    let mut pairs = Vec::new();

    for (pair, data) in resp.result {
        if let Some(arr) = data["c"].as_array() {
            if let Some(price_str) = arr[0].as_str() {
                if let Ok(price) = price_str.parse::<f64>() {
                    if let Some((base, quote)) = normalize_kraken_pair(&pair) {
                        pairs.push(PairPrice {
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
    Ok(pairs)
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

pub async fn fetch_bybit() -> Result<Vec<PairPrice>, Error> {
    let url = "https://api.bybit.com/v5/market/tickers?category=spot";
    let resp: BybitResponse = reqwest::get(url).await?.json().await?;
    let mut pairs = Vec::new();

    for t in resp.result.list {
        if let Ok(price) = t.lastPrice.parse::<f64>() {
            if let Some((base, quote)) = split_symbol(&t.symbol) {
                pairs.push(PairPrice {
                    base,
                    quote,
                    price,
                    is_spot: true,
                });
            }
        }
    }
    Ok(pairs)
}

/// --------------------
/// Helpers
/// --------------------
fn split_symbol(symbol: &str) -> Option<(String, String)> {
    // Works for Binance/Bybit-style "BTCUSDT"
    let bases = ["USDT", "BTC", "ETH", "USD", "EUR"];
    for q in bases {
        if symbol.ends_with(q) {
            let base = symbol.trim_end_matches(q).to_string();
            return Some((base, q.to_string()));
        }
    }
    None
            }
