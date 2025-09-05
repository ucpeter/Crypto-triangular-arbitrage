use crate::models::PairPrice;
use reqwest::Client;
use serde_json::Value;
use std::time::Duration;
use futures::future::join_all;

/// Build a reqwest client with timeout and common UA
fn make_client() -> Result<Client, String> {
    Client::builder()
        .timeout(Duration::from_secs(10))
        .user_agent("triangular-arb-scanner/1.0")
        .build()
        .map_err(|e| format!("client build error: {}", e))
}

/// Try to split typical exchange symbols into (base, quote)
/// We check longer quote tickers first to avoid false matches.
fn split_symbol(symbol: &str) -> Option<(String, String)> {
    // order matters: longest first
    let qlist = ["USDT", "USDC", "BUSD", "BTC", "ETH", "BNB", "USD", "EUR", "TRY"];
    let sym = symbol.to_uppercase();
    for q in qlist.iter() {
        if sym.ends_with(q) {
            let base = sym.trim_end_matches(q).to_string();
            if base.is_empty() {
                continue;
            }
            return Some((base, q.to_string()));
        }
    }
    // If no match, try common delimiters _ or - or / then split accordingly
    if let Some((a,b)) = symbol.split_once('_') {
        return Some((a.to_uppercase(), b.to_uppercase()));
    }
    if let Some((a,b)) = symbol.split_once('-') {
        return Some((a.to_uppercase(), b.to_uppercase()));
    }
    if let Some((a,b)) = symbol.split_once('/') {
        return Some((a.to_uppercase(), b.to_uppercase()));
    }
    None
}

/// Fetch many exchanges concurrently. Returns (exchange_name, vec of PairPrice).
/// This function is used by routes.rs
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
                Ok(vec) => {
                    tracing::info!(exchange = %name, count = vec.len(), "fetch complete");
                    out.push((name, vec));
                }
                Err(err) => {
                    tracing::error!(exchange = %name, error = %err, "fetch failed");
                    out.push((name, Vec::new()));
                }
            }
        } else {
            // task join error
            tracing::error!("fetch task join failed");
        }
    }
    out
}

/// Top-level single-exchange dispatcher used internally
pub async fn fetch_exchange_data(exchange: &str) -> Result<Vec<PairPrice>, String> {
    let client = make_client()?;
    match exchange.to_lowercase().as_str() {
        "binance" => fetch_binance(&client).await,
        "kucoin" => fetch_kucoin(&client).await,
        "bybit" => fetch_bybit(&client).await,
        "gate" | "gateio" => fetch_gateio(&client).await,
        "kraken" => fetch_kraken(&client).await,
        other => Err(format!("unsupported exchange: {}", other)),
    }
}

/* ------------------- per-exchange fetchers ------------------- */

async fn fetch_binance(client: &Client) -> Result<Vec<PairPrice>, String> {
    tracing::info!("fetching binance");
    // Use the lightweight ticker endpoint (gives symbol & price)
    let url = "https://api.binance.com/api/v3/ticker/price";
    let resp = client.get(url).send().await.map_err(|e| format!("binance http error: {}", e))?;
    if !resp.status().is_success() {
        return Err(format!("binance status: {}", resp.status()));
    }
    let arr: Vec<Value> = resp.json().await.map_err(|e| format!("binance json error: {}", e))?;
    let mut out = Vec::with_capacity(arr.len());
    for item in arr {
        if let (Some(sym), Some(p)) = (item.get("symbol").and_then(|v| v.as_str()), item.get("price").and_then(|v| v.as_str())) {
            if let Ok(price) = p.parse::<f64>() {
                if let Some((base, quote)) = split_symbol(sym) {
                    out.push(PairPrice { base: base.to_uppercase(), quote: quote.to_uppercase(), price, is_spot: true });
                }
            }
        }
    }
    tracing::info!(pairs = out.len(), "binance returned pairs");
    Ok(out)
}

async fn fetch_kucoin(client: &Client) -> Result<Vec<PairPrice>, String> {
    tracing::info!("fetching kucoin");
    let url = "https://api.kucoin.com/api/v1/market/allTickers";
    let resp = client.get(url).send().await.map_err(|e| format!("kucoin http error: {}", e))?;
    if !resp.status().is_success() {
        return Err(format!("kucoin status: {}", resp.status()));
    }
    let v: Value = resp.json().await.map_err(|e| format!("kucoin json error: {}", e))?;
    let mut out = Vec::new();
    // Some Kucoin responses have data.ticker array
    if let Some(arr) = v.get("data").and_then(|d| d.get("ticker")).and_then(|t| t.as_array()) {
        for item in arr {
            if let (Some(sym), Some(last)) = (item.get("symbol").and_then(|s| s.as_str()), item.get("last").and_then(|s| s.as_str())) {
                if let Ok(price) = last.parse::<f64>() {
                    if let Some((base, quote)) = sym.split_once('-') {
                        out.push(PairPrice { base: base.to_uppercase(), quote: quote.to_uppercase(), price, is_spot: true });
                    } else if let Some((b,q)) = split_symbol(sym) {
                        out.push(PairPrice { base: b.to_uppercase(), quote: q.to_uppercase(), price, is_spot: true });
                    }
                }
            }
        }
    }
    tracing::info!(pairs = out.len(), "kucoin returned pairs");
    Ok(out)
}

async fn fetch_bybit(client: &Client) -> Result<Vec<PairPrice>, String> {
    tracing::info!("fetching bybit");
    // v5 public tickers for spot
    let url = "https://api.bybit.com/v5/market/tickers?category=spot";
    let resp = client.get(url).send().await.map_err(|e| format!("bybit http error: {}", e))?;
    if !resp.status().is_success() {
        return Err(format!("bybit status: {}", resp.status()));
    }
    let v: Value = resp.json().await.map_err(|e| format!("bybit json error: {}", e))?;
    let mut out = Vec::new();
    if let Some(list) = v.get("result").and_then(|r| r.get("list")).and_then(|l| l.as_array()) {
        for item in list {
            // bybit v5 list item fields vary; try common keys
            let sym = item.get("symbol").and_then(|s| s.as_str()).or_else(|| item.get("symbolName").and_then(|s| s.as_str()));
            let price_s = item.get("lastPrice").and_then(|p| p.as_str()).or_else(|| item.get("last").and_then(|p| p.as_str()));
            if let (Some(sym), Some(price_s)) = (sym, price_s) {
                if let Ok(price) = price_s.parse::<f64>() {
                    if let Some((base, quote)) = split_symbol(sym) {
                        out.push(PairPrice { base: base.to_uppercase(), quote: quote.to_uppercase(), price, is_spot: true });
                    }
                }
            }
        }
    }
    tracing::info!(pairs = out.len(), "bybit returned pairs");
    Ok(out)
}

async fn fetch_gateio(client: &Client) -> Result<Vec<PairPrice>, String> {
    tracing::info!("fetching gateio");
    let url = "https://api.gate.io/api/v4/spot/tickers";
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
    tracing::info!(pairs = out.len(), "gateio returned pairs");
    Ok(out)
}

async fn fetch_kraken(client: &Client) -> Result<Vec<PairPrice>, String> {
    tracing::info!("fetching kraken");
    // Kraken has weird pair names; limit request to fewer pairs to be safe
    let url = "https://api.kraken.com/0/public/Ticker?pair=BTCUSD,ETHUSD,ETHXBT,XRPUSD";
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
                    // Normalize XBT -> BTC, slice as best-effort
                    let raw = raw_pair.replace("XBT", "BTC");
                    let norm = raw.to_uppercase();
                    if norm.len() >= 6 {
                        let (a, b) = norm.split_at(norm.len()-3);
                        out.push(PairPrice { base: a.to_string(), quote: b.to_string(), price, is_spot: true });
                    }
                }
            }
        }
    }
    tracing::info!(pairs = out.len(), "kraken returned pairs");
    Ok(out)
                }
