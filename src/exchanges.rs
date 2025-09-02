use std::collections::{HashMap, HashSet};
use reqwest::Client;
use serde_json::Value;

pub type PriceMap = HashMap<String, f64>;

/// Helper to standardize pair strings like "BTC/USDT"
fn format_pair(base: &str, quote: &str) -> String {
    format!("{}/{}", base.to_uppercase(), quote.to_uppercase())
}

/// Fetch Binance spot pairs and prices
pub async fn fetch_binance(client: &Client) -> (PriceMap, HashSet<String>) {
    let mut prices = PriceMap::new();
    let mut pairs = HashSet::new();

    if let Ok(resp) = client
        .get("https://api.binance.com/api/v3/exchangeInfo")
        .send()
        .await
    {
        if let Ok(data) = resp.json::<Value>().await {
            if let Some(symbols) = data.get("symbols").and_then(|s| s.as_array()) {
                for s in symbols {
                    if let (Some(base), Some(quote)) = (s.get("baseAsset"), s.get("quoteAsset")) {
                        pairs.insert(format_pair(base.as_str().unwrap(), quote.as_str().unwrap()));
                    }
                }
            }
        }
    }

    if let Ok(resp) = client
        .get("https://api.binance.com/api/v3/ticker/price")
        .send()
        .await
    {
        if let Ok(data) = resp.json::<Vec<Value>>().await {
            for item in data {
                if let (Some(symbol), Some(price)) =
                    (item.get("symbol"), item.get("price").and_then(|p| p.as_str()))
                {
                    // Try to split into base/quote for pair formatting
                    let (base, quote) = symbol.as_str().unwrap().split_at(symbol.as_str().unwrap().len() - 4);
                    if let Ok(p) = price.parse::<f64>() {
                        prices.insert(format_pair(base, quote), p);
                    }
                }
            }
        }
    }

    (prices, pairs)
}

/// KuCoin fetcher
pub async fn fetch_kucoin(client: &Client) -> (PriceMap, HashSet<String>) {
    let mut prices = PriceMap::new();
    let mut pairs = HashSet::new();

    if let Ok(resp) = client
        .get("https://api.kucoin.com/api/v1/symbols")
        .send()
        .await
    {
        if let Ok(data) = resp.json::<Value>().await {
            if let Some(symbols) = data.get("data").and_then(|s| s.as_array()) {
                for s in symbols {
                    if let (Some(base), Some(quote)) = (s.get("baseCurrency"), s.get("quoteCurrency")) {
                        pairs.insert(format_pair(base.as_str().unwrap(), quote.as_str().unwrap()));
                    }
                }
            }
        }
    }

    if let Ok(resp) = client
        .get("https://api.kucoin.com/api/v1/market/allTickers")
        .send()
        .await
    {
        if let Ok(data) = resp.json::<Value>().await {
            if let Some(tickers) = data["data"]["ticker"].as_array() {
                for t in tickers {
                    if let (Some(symbol), Some(price)) = (t.get("symbol"), t.get("last")) {
                        let parts: Vec<&str> = symbol.as_str().unwrap().split('-').collect();
                        if parts.len() == 2 {
                            if let Ok(p) = price.as_str().unwrap().parse::<f64>() {
                                prices.insert(format_pair(parts[0], parts[1]), p);
                            }
                        }
                    }
                }
            }
        }
    }

    (prices, pairs)
}

/// Bybit fetcher
pub async fn fetch_bybit(client: &Client) -> (PriceMap, HashSet<String>) {
    let mut prices = PriceMap::new();
    let mut pairs = HashSet::new();

    if let Ok(resp) = client
        .get("https://api.bybit.com/v5/market/instruments-info?category=spot")
        .send()
        .await
    {
        if let Ok(data) = resp.json::<Value>().await {
            if let Some(list) = data["result"]["list"].as_array() {
                for l in list {
                    if let (Some(base), Some(quote)) = (l.get("baseCoin"), l.get("quoteCoin")) {
                        pairs.insert(format_pair(base.as_str().unwrap(), quote.as_str().unwrap()));
                    }
                }
            }
        }
    }

    if let Ok(resp) = client
        .get("https://api.bybit.com/v5/market/tickers?category=spot")
        .send()
        .await
    {
        if let Ok(data) = resp.json::<Value>().await {
            if let Some(list) = data["result"]["list"].as_array() {
                for l in list {
                    if let (Some(symbol), Some(price)) = (l.get("symbol"), l.get("lastPrice")) {
                        let parts: Vec<&str> = symbol.as_str().unwrap().split('/').collect();
                        if parts.len() == 2 {
                            if let Ok(p) = price.as_str().unwrap().parse::<f64>() {
                                prices.insert(format_pair(parts[0], parts[1]), p);
                            }
                        }
                    }
                }
            }
        }
    }

    (prices, pairs)
}

/// Gate.io fetcher
pub async fn fetch_gateio(client: &Client) -> (PriceMap, HashSet<String>) {
    let mut prices = PriceMap::new();
    let mut pairs = HashSet::new();

    if let Ok(resp) = client
        .get("https://api.gateio.ws/api/v4/spot/currency_pairs")
        .send()
        .await
    {
        if let Ok(data) = resp.json::<Vec<Value>>().await {
            for item in data {
                if let (Some(base), Some(quote)) = (item.get("base"), item.get("quote")) {
                    pairs.insert(format_pair(base.as_str().unwrap(), quote.as_str().unwrap()));
                }
            }
        }
    }

    if let Ok(resp) = client
        .get("https://api.gateio.ws/api/v4/spot/tickers")
        .send()
        .await
    {
        if let Ok(data) = resp.json::<Vec<Value>>().await {
            for item in data {
                if let (Some(symbol), Some(price)) = (item.get("currency_pair"), item.get("last")) {
                    let parts: Vec<&str> = symbol.as_str().unwrap().split('_').collect();
                    if parts.len() == 2 {
                        if let Ok(p) = price.as_str().unwrap().parse::<f64>() {
                            prices.insert(format_pair(parts[0], parts[1]), p);
                        }
                    }
                }
            }
        }
    }

    (prices, pairs)
}

/// Kraken fetcher
pub async fn fetch_kraken(client: &Client) -> (PriceMap, HashSet<String>) {
    let mut prices = PriceMap::new();
    let mut pairs = HashSet::new();

    if let Ok(resp) = client
        .get("https://api.kraken.com/0/public/AssetPairs")
        .send()
        .await
    {
        if let Ok(data) = resp.json::<Value>().await {
            if let Some(result) = data["result"].as_object() {
                for (symbol, pair) in result {
                    if let (Some(base), Some(quote)) = (pair.get("base"), pair.get("quote")) {
                        pairs.insert(format_pair(base.as_str().unwrap(), quote.as_str().unwrap()));
                    }
                }
            }
        }
    }

    if let Ok(resp) = client
        .get("https://api.kraken.com/0/public/Ticker?pair=BTCUSD,ETHUSD")
        .send()
        .await
    {
        if let Ok(data) = resp.json::<Value>().await {
            if let Some(result) = data["result"].as_object() {
                for (symbol, ticker) in result {
                    if let Some(price) = ticker["c"][0].as_str() {
                        if let Ok(p) = price.parse::<f64>() {
                            // Simplified; use actual base/quote parsing logic if needed
                            prices.insert(symbol.to_uppercase(), p);
                        }
                    }
                }
            }
        }
    }

    (prices, pairs)
                                                                }
