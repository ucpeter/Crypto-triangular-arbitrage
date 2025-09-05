use crate::models::PairPrice;
use reqwest::Client;
use serde_json::Value;
use std::time::Duration;
use futures::future::join_all;

/// Fetch data for a single exchange by name (lowercase).
/// Returns Vec<PairPrice> (may be empty on error).
pub async fn fetch_exchange_data(exchange: &str) -> Result<Vec<PairPrice>, String> {
    // single client with timeout
    let client = Client::builder()
        .timeout(Duration::from_secs(10))
        .user_agent("triangular-arb-scanner/1.0")
        .build()
        .map_err(|e| format!("client build error: {}", e))?;

    match exchange.to_lowercase().as_str() {
        "binance" => fetch_binance(&client).await,
        "kucoin" => fetch_kucoin(&client).await,
        "bybit" => fetch_bybit(&client).await,
        "gate" | "gateio" => fetch_gateio(&client).await,
        "kraken" => fetch_kraken(&client).await,
        other => Err(format!("unsupported exchange: {}", other)),
    }
}

/// Fetch many exchanges concurrently. Returns Vec<(name, pairs)>.
pub async fn fetch_many(exchanges: &[String]) -> Vec<(String, Vec<PairPrice>)> {
    let mut handles = Vec::new();
    for ex in exchanges.iter() {
        let name = ex.clone();
        handles.push(tokio::spawn(async move {
            let res = fetch_exchange_data(&name).await;
            (name, res)
        }));
    }

    let mut out = Vec::new();
    let results = join_all(handles).await;
    for r in results {
        if let Ok((name, res)) = r {
            match res {
                Ok(vec) => out.push((name, vec)),
                Err(err) => {
                    tracing::error!("fetch {} failed: {}", name, err);
                    out.push((name, Vec::new()));
                }
            }
        }
    }
    out
}

/* ---------- exchange fetchers ---------- */

async fn fetch_binance(client: &Client) -> Result<Vec<PairPrice>, String> {
    let url = "https://api.binance.com/api/v3/ticker/price";
    tracing::info!("fetching binance");
    let resp = client.get(url).send().await.map_err(|e| format!("binance http error: {}", e))?;
    if !resp.status().is_success() {
        return Err(format!("binance status: {}", resp.status()));
    }
    let arr: Vec<Value> = resp.json().await.map_err(|e| format!("binance json error: {}", e))?;
    let mut out = Vec::new();
    for item in arr {
        if let (Some(sym), Some(p)) = (item.get("symbol").and_then(|v| v.as_str()), item.get("price").and_then(|v| v.as_str())) {
            if let Ok(price) = p.parse::<f64>() {
                if let Some((base, quote)) = split_symbol(sym) {
                    out.push(PairPrice { base, quote, price, is_spot: true });
                }
            }
        }
    }
    tracing::info!("binance returned {} pairs", out.len());
    Ok(out)
}

async fn fetch_kucoin(client: &Client) -> Result<Vec<PairPrice>, String> {
    let url = "https://api.kucoin.com/api/v1/market/allTickers";
    tracing::info!("fetching kucoin");
    let resp = client.get(url).send().await.map_err(|e| format!("kucoin http error: {}", e))?;
    if !resp.status().is_success() {
        return Err(format!("kucoin status: {}", resp.status()));
    }
    let v: Value = resp.json().await.map_err(|e| format!("kucoin json error: {}", e))?;
    let mut out = Vec::new();
    if let Some(arr) = v.get("data").and_then(|d| d.get("ticker")).and_then(|t| t.as_array()) {
        for item in arr {
            if let (Some(sym), Some(last)) = (item.get("symbol").and_then(|s| s.as_str()), item.get("last").and_then(|s| s.as_str())) {
                if let Ok(price) = last.parse::<f64>() {
                    if let Some((base, quote)) = sym.split_once('-') {
                        out.push(PairPrice { base: base.to_uppercase(), quote: quote.to_uppercase(), price, is_spot: true });
                    }
                }
            }
        }
    }
    tracing::info!("kucoin returned {} pairs", out.len());
    Ok(out)
}

async fn fetch_bybit(client: &Client) -> Result<Vec<PairPrice>, String> {
    let url = "https://api.bybit.com/v5/market/tickers?category=spot";
    tracing::info!("fetching bybit");
    let resp = client.get(url).send().await.map_err(|e| format!("bybit http error: {}", e))?;
    if !resp.status().is_success() {
        return Err(format!("bybit status: {}", resp.status()));
    }
    let v: Value = resp.json().await.map_err(|e| format!("bybit json error: {}", e))?;
    let mut out = Vec::new();
    if let Some(list) = v.get("result").and_then(|r| r.get("list")).and_then(|l| l.as_array()) {
        for item in list {
            if let (Some(sym), Some(last)) = (item.get("symbol").and_then(|s| s.as_str()), item.get("lastPrice").and_then(|s| s.as_str())) {
                if let Ok(price) = last.parse::<f64>() {
                    if let Some((base, quote)) = split_symbol(sym) {
                        out.push(PairPrice { base, quote, price, is_spot: true });
                    }
                }
            }
        }
    }
    tracing::info!("bybit returned {} pairs", out.len());
    Ok(out)
}

async fn fetch_gateio(client: &Client) -> Result<Vec<PairPrice>, String> {
    let url = "https://api.gateio.ws/api/v4/spot/tickers";
    tracing::info!("fetching gateio");
    let resp = client.get(url).send().await.map_err(|e| format!("gateio http error: {}", e))?;
    if !resp.status().is_success() {
        return Err(format!("gateio status: {}", resp.status()));
    }
    let arr: Vec<Value> = resp.json().await.map_err(|e| format!("gateio json error: {}", e))?;
    let mut out = Vec::new();
    for item in arr {
        if let (Some(pair), Some(last)) = (item.get("currency_pair").and_then(|s| s.as_str()), item.get("last").and_then(|s| s.as_str())) {
            if let Ok(price) = last.parse::<f64>() {
                if let Some((base, quote)) = pair.split_once('_') {
                    out.push(PairPrice { base: base.to_uppercase(), quote: quote.to_uppercase(), price, is_spot: true });
                }
            }
        }
    }
    tracing::info!("gateio returned {} pairs", out.len());
    Ok(out)
}

async fn fetch_kraken(client: &Client) -> Result<Vec<PairPrice>, String> {
    // keep it limited to a few common pairs to avoid crazy mapping logic
    let url = "https://api.kraken.com/0/public/Ticker?pair=BTCUSD,ETHUSD";
    tracing::info!("fetching kraken");
    let resp = client.get(url).send().await.map_err(|e| format!("kraken http error: {}", e))?;
    if !resp.status().is_success() {
        return Err(format!("kraken status: {}", resp.status()));
    }
    let v: Value = resp.json().await.map_err(|e| format!("kraken json error: {}", e))?;
    let mut out = Vec::new();
    if let Some(result) = v.get("result").and_then(|r| r.as_object()) {
        for (raw_pair, info) in result {
            if let Some(price_str) = info.get("c").and_then(|c| c.get(0)).and_then(|p| p.as_str()) {
                if let Ok(price) = price_str.parse::<f64>() {
                    let (base, quote) = normalize_kraken_pair(raw_pair);
                    out.push(PairPrice { base, quote, price, is_spot: true });
                }
            }
        }
    }
    tracing::info!("kraken returned {} pairs", out.len());
    Ok(out)
}

/* helpers */

fn split_symbol(symbol: &str) -> Option<(String, String)> {
    // try common quote endings
    let qlist = ["USDT", "USDC", "BTC", "ETH", "BUSD", "USD", "EUR"];
    for q in qlist.iter() {
        if symbol.ends_with(q) {
            let base = symbol.trim_end_matches(q).to_string();
            return Some((base.to_uppercase(), q.to_string()));
        }
    }
    None
}

fn normalize_kraken_pair(raw: &str) -> (String, String) {
    // small heuristic; Kraken pair names are odd—this keeps it simple
    let s = raw.to_uppercase();
    if s.contains("XBT") || s.contains("BTC") {
        if s.contains("USD") {
            return ("BTC".to_string(), "USD".to_string());
        }
    }
    // fallback split
    if s.len() > 3 {
        let (a, b) = s.split_at(s.len() - 3);
        return (a.to_string(), b.to_string());
    }
    (s, "USD".to_string())
    }
