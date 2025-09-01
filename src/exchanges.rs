use reqwest::Client;
use serde_json::Value;
use std::collections::HashMap;

pub type PriceMap = HashMap<String, f64>; // "BASE/QUOTE" -> price

// Known quotes (longer first to avoid USDT vs USD parsing ambiguity)
fn known_quotes() -> &'static [&'static str] {
    &[
        "USDT","USDC","BUSD","USD","TUSD","FDUSD","DAI","EUR","BTC","ETH","BNB","TRY","GBP","AUD","BRL","IDRT","NGN","UAH","RUB","JPY","KRW","ZAR","SAR","AED"
    ]
}

fn normalize_pair_raw(sym: &str) -> Option<(String, String)> {
    // handles formats: "BTCUSDT", "BTC-USDT", "BTC_USDT", "BTC/USDT"
    if sym.contains('/') {
        let mut sp = sym.split('/');
        return Some((sp.next()?.to_string(), sp.next()?.to_string()));
    }
    if sym.contains('-') {
        let mut sp = sym.split('-');
        return Some((sp.next()?.to_string(), sp.next()?.to_string()));
    }
    if sym.contains('_') {
        let mut sp = sym.split('_');
        return Some((sp.next()?.to_string(), sp.next()?.to_string()));
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

fn to_key(base: &str, quote: &str) -> String {
    format!("{}/{}", base.to_uppercase(), quote.to_uppercase())
}

// ---------- BINANCE ----------
pub async fn fetch_prices_binance() -> PriceMap {
    let url = "https://api.binance.com/api/v3/ticker/price";
    let client = Client::new();
    let mut map = PriceMap::new();

    if let Ok(resp) = client.get(url).send().await {
        if let Ok(json) = resp.json::<Vec<Value>>().await {
            for e in json {
                if let (Some(sym), Some(p)) = (
                    e.get("symbol").and_then(|v| v.as_str()),
                    e.get("price").and_then(|v| v.as_str()).and_then(|s| s.parse::<f64>().ok()),
                ) {
                    if let Some((b, q)) = normalize_pair_raw(sym) {
                        map.insert(to_key(&b, &q), p);
                    }
                }
            }
        }
    }
    map
}

// ---------- KUCOIN ----------
pub async fn fetch_prices_kucoin() -> PriceMap {
    let url = "https://api.kucoin.com/api/v1/market/allTickers";
    let client = Client::new();
    let mut map = PriceMap::new();

    if let Ok(resp) = client.get(url).send().await {
        if let Ok(json) = resp.json::<Value>().await {
            if let Some(arr) = json.get("data").and_then(|d| d.get("ticker")).and_then(|t| t.as_array()) {
                for e in arr {
                    if let (Some(sym), Some(p)) = (
                        e.get("symbol").and_then(|v| v.as_str()),
                        e.get("last").and_then(|v| v.as_str()).and_then(|s| s.parse::<f64>().ok()),
                    ) {
                        if let Some((b, q)) = normalize_pair_raw(sym) {
                            map.insert(to_key(&b, &q), p);
                        }
                    }
                }
            }
        }
    }
    map
}

// ---------- BYBIT (v5 spot) ----------
pub async fn fetch_prices_bybit() -> PriceMap {
    let url = "https://api.bybit.com/v5/market/tickers?category=spot";
    let client = Client::new();
    let mut map = PriceMap::new();

    if let Ok(resp) = client.get(url).send().await {
        if let Ok(json) = resp.json::<Value>().await {
            if let Some(arr) = json.get("result").and_then(|r| r.get("list")).and_then(|l| l.as_array()) {
                for e in arr {
                    if let (Some(sym), Some(p)) = (
                        e.get("symbol").and_then(|v| v.as_str()),
                        e.get("lastPrice").and_then(|v| v.as_str()).and_then(|s| s.parse::<f64>().ok()),
                    ) {
                        if let Some((b, q)) = normalize_pair_raw(sym) {
                            map.insert(to_key(&b, &q), p);
                        }
                    }
                }
            }
        }
    }
    map
}

// ---------- GATE.IO ----------
pub async fn fetch_prices_gateio() -> PriceMap {
    let url = "https://api.gateio.ws/api/v4/spot/tickers";
    let client = Client::new();
    let mut map = PriceMap::new();

    if let Ok(resp) = client.get(url).send().await {
        if let Ok(json) = resp.json::<Vec<Value>>().await {
            for e in json {
                if let (Some(sym), Some(p)) = (
                    e.get("currency_pair").and_then(|v| v.as_str()),
                    e.get("last").and_then(|v| v.as_str()).and_then(|s| s.parse::<f64>().ok()),
                ) {
                    if let Some((b, q)) = normalize_pair_raw(sym) {
                        map.insert(to_key(&b, &q), p);
                    }
                }
            }
        }
    }
    map
}

// ---------- KRAKEN ----------
// We resolve AssetPairs first to get "altname" and then query Ticker in chunks.
pub async fn fetch_prices_kraken() -> PriceMap {
    let client = Client::new();
    let mut map = PriceMap::new();

    // 1) AssetPairs
    let pairs_url = "https://api.kraken.com/0/public/AssetPairs";
    let mut key_to_alt: HashMap<String, String> = HashMap::new();

    if let Ok(resp) = client.get(pairs_url).send().await {
        if let Ok(json) = resp.json::<Value>().await {
            if let Some(obj) = json.get("result").and_then(|r| r.as_object()) {
                for (key, val) in obj {
                    if let Some(alt) = val.get("altname").and_then(|v| v.as_str()) {
                        key_to_alt.insert(key.clone(), alt.to_string());
                    }
                }
            }
        }
    }

    if key_to_alt.is_empty() {
        return map;
    }

    // 2) Chunk Ticker requests (Kraken allows many, but we chunk to avoid URL length issues)
    let keys: Vec<String> = key_to_alt.keys().cloned().collect();
    const CHUNK: usize = 150;
    for chunk in keys.chunks(CHUNK) {
        let joined = chunk.join(",");
        let url = format!("https://api.kraken.com/0/public/Ticker?pair={}", joined);
        if let Ok(resp) = client.get(&url).send().await {
            if let Ok(json) = resp.json::<Value>().await {
                if let Some(obj) = json.get("result").and_then(|r| r.as_object()) {
                    for (k, data) in obj {
                        if let Some(last) = data.get("c").and_then(|arr| arr.get(0)).and_then(|v| v.as_str()).and_then(|s| s.parse::<f64>().ok()) {
                            if let Some(alt) = key_to_alt.get(k) {
                                if let Some((b, q)) = normalize_pair_raw(alt) {
                                    map.insert(to_key(&b, &q), last);
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
