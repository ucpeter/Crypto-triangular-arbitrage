use crate::models::PairPrice;
use reqwest::Client;
use serde::Deserialize;
use std::collections::HashMap;
use tokio::task;
use tracing::{info, error};

/// Fetch many exchanges concurrently
pub async fn fetch_many(exchanges: Vec<String>) -> HashMap<String, Vec<PairPrice>> {
    let mut results: HashMap<String, Vec<PairPrice>> = HashMap::new();
    let mut tasks = Vec::new();

    for ex in exchanges {
        tasks.push(task::spawn(async move {
            let data = fetch_exchange_data(&ex).await.unwrap_or_default();
            (ex, data)
        }));
    }

    for t in tasks {
        if let Ok((ex, data)) = t.await {
            info!("fetch complete exchange={} count={}", ex, data.len());
            results.insert(ex, data);
        }
    }

    results
}

/// Dispatch function for supported exchanges
pub async fn fetch_exchange_data(exchange: &str) -> Result<Vec<PairPrice>, reqwest::Error> {
    match exchange.to_lowercase().as_str() {
        "binance" => fetch_binance().await,
        "kucoin" => fetch_kucoin().await,
        "bybit" => fetch_bybit().await,
        "gateio" | "gate" => fetch_gateio().await,
        _ => Ok(Vec::new()),
    }
}

///////////////////////////////////////
/// BINANCE
///////////////////////////////////////
#[derive(Debug, Deserialize)]
struct BinanceTicker {
    symbol: String,
    price: String,
}

async fn fetch_binance() -> Result<Vec<PairPrice>, reqwest::Error> {
    let url = "https://api.binance.com/api/v3/ticker/price";
    let resp: Vec<BinanceTicker> = Client::new().get(url).send().await?.json().await?;

    let mut out = Vec::new();
    for t in resp {
        if t.symbol.len() < 6 {
            continue;
        }

        // Try to split symbol into base/quote using known quote suffixes
        let known_quotes = ["USDT", "BUSD", "BTC", "ETH", "BNB"];
        let mut base = None;
        let mut quote = None;

        for q in &known_quotes {
            if t.symbol.ends_with(q) {
                let b = t.symbol.trim_end_matches(q);
                base = Some(b.to_string());
                quote = Some(q.to_string());
                break;
            }
        }

        if let (Some(b), Some(q)) = (base, quote) {
            if let Ok(p) = t.price.parse::<f64>() {
                out.push(PairPrice {
                    base: b,
                    quote: q,
                    price: p,
                    is_spot: true,
                });
            }
        }
    }

    info!("binance returned pairs pairs={}", out.len());
    Ok(out)
}

///////////////////////////////////////
/// KUCOIN
///////////////////////////////////////
#[derive(Debug, Deserialize)]
struct KucoinTicker {
    symbol: String,
    price: String,
}

#[derive(Debug, Deserialize)]
struct KucoinResp {
    data: Vec<KucoinTicker>,
}

async fn fetch_kucoin() -> Result<Vec<PairPrice>, reqwest::Error> {
    let url = "https://api.kucoin.com/api/v1/market/allTickers";
    let resp: KucoinResp = Client::new().get(url).send().await?.json().await?;

    let mut out = Vec::new();
    for t in resp.data {
        if let Some((b, q)) = t.symbol.split_once("-") {
            if let Ok(p) = t.price.parse::<f64>() {
                out.push(PairPrice {
                    base: b.to_string(),
                    quote: q.to_string(),
                    price: p,
                    is_spot: true,
                });
            }
        }
    }

    info!("kucoin returned pairs pairs={}", out.len());
    Ok(out)
}

///////////////////////////////////////
/// BYBIT
///////////////////////////////////////
#[derive(Debug, Deserialize)]
struct BybitTicker {
    symbol: String,
    lastPrice: String,
}

#[derive(Debug, Deserialize)]
struct BybitResp {
    result: Vec<BybitTicker>,
}

async fn fetch_bybit() -> Result<Vec<PairPrice>, reqwest::Error> {
    let url = "https://api.bybit.com/v5/market/tickers?category=spot";
    let resp: BybitResp = Client::new().get(url).send().await?.json().await?;

    let mut out = Vec::new();
    for t in resp.result {
        // Bybit symbols like "ETHUSDT"
        let known_quotes = ["USDT", "BTC", "ETH", "USDC"];
        let mut base = None;
        let mut quote = None;

        for q in &known_quotes {
            if t.symbol.ends_with(q) {
                let b = t.symbol.trim_end_matches(q);
                base = Some(b.to_string());
                quote = Some(q.to_string());
                break;
            }
        }

        if let (Some(b), Some(q)) = (base, quote) {
            if let Ok(p) = t.lastPrice.parse::<f64>() {
                out.push(PairPrice {
                    base: b,
                    quote: q,
                    price: p,
                    is_spot: true,
                });
            }
        }
    }

    info!("bybit returned pairs pairs={}", out.len());
    Ok(out)
}

///////////////////////////////////////
/// GATE.IO
///////////////////////////////////////
#[derive(Debug, Deserialize)]
struct GateTicker {
    currency_pair: String,
    last: String,
}

async fn fetch_gateio() -> Result<Vec<PairPrice>, reqwest::Error> {
    let url = "https://api.gate.io/api/v4/spot/tickers";
    let resp: Vec<GateTicker> = Client::new().get(url).send().await?.json().await?;

    let mut out = Vec::new();
    for t in resp {
        if let Some((b, q)) = t.currency_pair.split_once("_") {
            if let Ok(p) = t.last.parse::<f64>() {
                out.push(PairPrice {
                    base: b.to_string(),
                    quote: q.to_string(),
                    price: p,
                    is_spot: true,
                });
            }
        }
    }

    info!("gateio returned pairs pairs={}", out.len());
    Ok(out)
    }
