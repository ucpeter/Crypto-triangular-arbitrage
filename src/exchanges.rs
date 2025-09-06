use crate::models::PairPrice;
use reqwest::Client;
use serde::Deserialize;
use std::error::Error;
use std::collections::HashMap;

//
// BINANCE
//
#[derive(Deserialize)]
struct BinanceSymbol {
    baseAsset: String,
    quoteAsset: String,
    status: String,
}

#[derive(Deserialize)]
struct BinanceExchangeInfo {
    symbols: Vec<BinanceSymbol>,
}

#[derive(Deserialize)]
struct BinancePrice {
    symbol: String,
    price: String,
}

pub async fn fetch_binance(client: &Client) -> Result<Vec<PairPrice>, Box<dyn Error>> {
    // Get symbol list
    let info_url = "https://api.binance.com/api/v3/exchangeInfo";
    let info: BinanceExchangeInfo = client.get(info_url).send().await?.json().await?;
    // Get all prices
    let prices_url = "https://api.binance.com/api/v3/ticker/price";
    let prices: Vec<BinancePrice> = client.get(prices_url).send().await?.json().await?;

    let mut map = std::collections::HashMap::new();
    for p in prices {
        if let Ok(v) = p.price.parse::<f64>() {
            map.insert(p.symbol, v);
        }
    }

    let mut out = Vec::new();
    for s in info.symbols {
        if s.status != "TRADING" {
            continue;
        }
        let symbol = format!("{}{}", s.baseAsset, s.quoteAsset);
        if let Some(price) = map.get(&symbol) {
            out.push(PairPrice {
                base: s.baseAsset.to_uppercase(),
                quote: s.quoteAsset.to_uppercase(),
                price: *price,
                is_spot: true,
            });
        }
    }

    Ok(out)
}

//
// KUCOIN
//
#[derive(Deserialize)]
struct KucoinTicker {
    symbol: String,
    price: String,
}

#[derive(Deserialize)]
struct KucoinResp {
    data: Vec<KucoinTicker>,
}

pub async fn fetch_kucoin(client: &Client) -> Result<Vec<PairPrice>, Box<dyn Error>> {
    let url = "https://api.kucoin.com/api/v1/market/allTickers";
    let resp: KucoinResp = client.get(url).send().await?.json().await?;
    let mut out = Vec::new();

    for t in resp.data {
        if let Some((base, quote)) = t.symbol.split_once('-') {
            if let Ok(price) = t.price.parse::<f64>() {
                out.push(PairPrice {
                    base: base.to_uppercase(),
                    quote: quote.to_uppercase(),
                    price,
                    is_spot: true,
                });
            }
        }
    }

    Ok(out)
}

//
// BYBIT
//
#[derive(Deserialize)]
struct BybitSymbol {
    baseCoin: String,
    quoteCoin: String,
    status: String,
}

#[derive(Deserialize)]
struct BybitSymbolResp {
    result: BybitSymbolResult,
}

#[derive(Deserialize)]
struct BybitSymbolResult {
    list: Vec<BybitSymbol>,
}

#[derive(Deserialize)]
struct BybitTicker {
    symbol: String,
    lastPrice: String,
}

#[derive(Deserialize)]
struct BybitTickerResp {
    result: Vec<BybitTicker>,
}

pub async fn fetch_bybit(client: &Client) -> Result<Vec<PairPrice>, Box<dyn Error>> {
    // Fetch symbol info
    let info_url = "https://api.bybit.com/v5/market/instruments-info?category=spot";
    let info: BybitSymbolResp = client.get(info_url).send().await?.json().await?;

    // Fetch tickers
    let tickers_url = "https://api.bybit.com/v5/market/tickers?category=spot";
    let tickers: BybitTickerResp = client.get(tickers_url).send().await?.json().await?;

    let mut map = std::collections::HashMap::new();
    for t in tickers.result {
        if let Ok(v) = t.lastPrice.parse::<f64>() {
            map.insert(t.symbol, v);
        }
    }

    let mut out = Vec::new();
    for s in info.result.list {
        if s.status != "Trading" {
            continue;
        }
        let symbol = format!("{}{}", s.baseCoin, s.quoteCoin);
        if let Some(price) = map.get(&symbol) {
            out.push(PairPrice {
                base: s.baseCoin.to_uppercase(),
                quote: s.quoteCoin.to_uppercase(),
                price: *price,
                is_spot: true,
            });
        }
    }

    Ok(out)
}

//
// GATE.IO
//
#[derive(Deserialize)]
struct GateTicker {
    currency_pair: String,
    last: String,
}

pub async fn fetch_gateio(client: &Client) -> Result<Vec<PairPrice>, Box<dyn Error>> {
    let url = "https://api.gateio.ws/api/v4/spot/tickers";
    let resp: Vec<GateTicker> = client.get(url).send().await?.json().await?;
    let mut out = Vec::new();

    for t in resp {
        if let Some((base, quote)) = t.currency_pair.split_once('_') {
            if let Ok(price) = t.last.parse::<f64>() {
                out.push(PairPrice {
                    base: base.to_uppercase(),
                    quote: quote.to_uppercase(),
                    price,
                    is_spot: true,
                });
            }
        }
    }

    Ok(out)
}
pub async fn fetch_many(exchanges: Vec<String>) -> HashMap<String, Vec<PairPrice>> {
    let mut results = HashMap::new();

    for ex in exchanges {
        match fetch_exchange_data(&ex).await {
            Ok(pairs) => {
                results.insert(ex.clone(), pairs);
            }
            Err(e) => {
                tracing::error!("❌ Error fetching data from {}: {:?}", ex, e);
                results.insert(ex.clone(), Vec::new());
            }
        }
    }

    results
}
