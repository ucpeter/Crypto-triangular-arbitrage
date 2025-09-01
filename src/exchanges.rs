use crate::models::PriceMap;
use reqwest::Client;
use serde_json::Value;
use std::collections::HashMap;

/// Normalize pair strings into BASE/QUOTE format
fn known_quotes() -> &'static [&'static str] {
    &[
        "USDT","USDC","BUSD","USD","DAI","BTC","ETH","BNB","EUR","GBP","TRY","AUD","BRL","JPY","KRW",
    ]
}

fn normalize_pair_raw(sym: &str) -> Option<(String,String)> {
    if sym.contains('/') {
        let mut s = sym.split('/');
        return Some((s.next()?.to_string(), s.next()?.to_string()));
    }
    if sym.contains('-') {
        let mut s = sym.split('-');
        return Some((s.next()?.to_string(), s.next()?.to_string()));
    }
    if sym.contains('_') {
        let mut s = sym.split('_');
        return Some((s.next()?.to_string(), s.next()?.to_string()));
    }
    let u = sym.to_uppercase();
    for q in known_quotes() {
        if u.ends_with(q) && u.len() > q.len() {
            let base = &u[..u.len() - q.len()];
            return Some((base.to_string(), (*q).to_string()));
        }
    }
    None
}

fn key(base: &str, quote: &str) -> String {
    format!("{}/{}", base.to_uppercase(), quote.to_uppercase())
}

async fn fetch_json(client: &Client, url: &str) -> Option<Value> {
    client.get(url).send().await.ok()?.json::<Value>().await.ok()
}

pub async fn fetch_binance() -> PriceMap {
    let client = Client::new();
    let mut map: PriceMap = HashMap::new();
    if let Some(Value::Array(arr)) = fetch_json(&client, "https://api.binance.com/api/v3/ticker/price").await {
        for e in arr {
            if let (Some(sym), Some(price_s)) = (e.get("symbol").and_then(|v| v.as_str()), e.get("price").and_then(|v| v.as_str())) {
                if let Ok(p) = price_s.parse::<f64>() {
                    if let Some((b,q)) = normalize_pair_raw(sym) {
                        map.insert(key(&b,&q), p);
                    }
                }
            }
        }
    }
    map
}

pub async fn fetch_kucoin() -> PriceMap {
    let client = Client::new();
    let mut map: PriceMap = HashMap::new();
    if let Some(json) = fetch_json(&client, "https://api.kucoin.com/api/v1/market/allTickers").await {
        if let Some(arr) = json.get("data").and_then(|d| d.get("ticker")).and_then(|t| t.as_array()) {
            for e in arr {
                if let (Some(sym), Some(last_s)) = (e.get("symbol").and_then(|v| v.as_str()), e.get("last").and_then(|v| v.as_str())) {
                    if let Ok(p) = last_s.parse::<f64>() {
                        if let Some((b,q)) = normalize_pair_raw(sym) {
                            map.insert(key(&b,&q), p);
                        }
                    }
                }
            }
        }
    }
    map
}

pub async fn fetch_bybit() -> PriceMap {
    let client = Client::new();
    let mut map: PriceMap = HashMap::new();
    if let Some(json) = fetch_json(&client, "https://api.bybit.com/v5/market/tickers?category=spot").await {
        if let Some(list) = json.get("result").and_then(|r| r.get("list")).and_then(|l| l.as_array()) {
            for e in list {
                if let (Some(sym), Some(last_s)) = (e.get("symbol").and_then(|v| v.as_str()), e.get("lastPrice").and_then(|v| v.as_str())) {
                    if let Ok(p) = last_s.parse::<f64>() {
                        if let Some((b,q)) = normalize_pair_raw(sym) {
                            map.insert(key(&b,&q), p);
                        }
                    }
                }
            }
        }
    }
    map
}

pub async fn fetch_gateio() -> PriceMap {
    let client = Client::new();
    let mut map: PriceMap = HashMap::new();
    if let Some(Value::Array(arr)) = fetch_json(&client, "https://api.gateio.ws/api/v4/spot/tickers").await {
        for e in arr {
            if let (Some(sym), Some(last_s)) = (e.get("currency_pair").and_then(|v| v.as_str()), e.get("last").and_then(|v| v.as_str())) {
                if let Ok(p) = last_s.parse::<f64>() {
                    if let Some((b,q)) = normalize_pair_raw(sym) {
                        map.insert(key(&b,&q), p);
                    }
                }
            }
        }
    }
    map
}

pub async fn fetch_kraken() -> PriceMap {
    let client = Client::new();
    let mut map: PriceMap = HashMap::new();

    if let Some(json) = fetch_json(&client, "https://api.kraken.com/0/public/AssetPairs").await {
        if let Some(obj) = json.get("result").and_then(|r| r.as_object()) {
            let mut alt: HashMap<String,String> = HashMap::new();
            for (k,v) in obj {
                if let Some(a) = v.get("altname").and_then(|x| x.as_str()) {
                    alt.insert(k.clone(), a.to_string());
                }
            }
            if !alt.is_empty() {
                let keys: Vec<String> = alt.keys().cloned().collect();
                const CHUNK: usize = 100;
                for chunk in keys.chunks(CHUNK) {
                    let joined = chunk.join(",");
                    let url = format!("https://api.kraken.com/0/public/Ticker?pair={}", joined);
                    if let Some(tjson) = fetch_json(&client, &url).await {
                        if let Some(res) = tjson.get("result").and_then(|r| r.as_object()) {
                            for (k, d) in res {
                                if let Some(last_s) = d.get("c").and_then(|arr| arr.get(0)).and_then(|v| v.as_str()) {
                                    if let Ok(p) = last_s.parse::<f64>() {
                                        if let Some(altname) = alt.get(k) {
                                            if let Some((b,q)) = normalize_pair_raw(altname) {
                                                map.insert(key(&b,&q), p);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    map
        }
