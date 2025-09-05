use crate::models::PairPrice;
use reqwest::Client;
use serde::Deserialize;
use std::error::Error;

//
// BINANCE
//
#[derive(Deserialize)]
struct BinanceSymbol {
    symbol: String,
    baseAsset: String,
    quoteAsset: String,
    status: String,
}

#[derive(Deserialize)]
struct BinanceExchangeInfo {
    symbols: Vec<BinanceSymbol>,
}

pub async fn fetch_binance(client: &Client) -> Result<Vec<PairPrice>, Box<dyn Error>> {
    let url = "https://api.binance.com/api/v3/exchangeInfo";
    let resp: BinanceExchangeInfo = client.get(url).send().await?.json().await?;
    let mut out = Vec::new();

    for s in resp.symbols {
        if s.status != "TRADING" {
            continue;
        }
        // Fetch latest price
        let ticker_url = format!("https://api.binance.com/api/v3/ticker/price?symbol={}", s.symbol);
        if let Ok(price_resp) = client.get(&ticker_url).send().await {
            if let Ok(p) = price_resp.json::<serde_json::Value>().await {
                if let Some(price) = p["price"].as_str().and_then(|x| x.parse::<f64>().ok()) {
                    out.push(PairPrice {
                        base: s.baseAsset.to_uppercase(),
                        quote: s.quoteAsset.to_uppercase(),
                        price,
                        is_spot: true,
                    });
                }
            }
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
struct BybitTicker {
    symbol: String,
    lastPrice: String,
}

#[derive(Deserialize)]
struct BybitResp {
    result: Vec<BybitTicker>,
}

pub async fn fetch_bybit(client: &Client) -> Result<Vec<PairPrice>, Box<dyn Error>> {
    let url = "https://api.bybit.com/v5/market/tickers?category=spot";
    let resp: BybitResp = client.get(url).send().await?.json().await?;
    let mut out = Vec::new();

    for t in resp.result {
        if let Some((base, quote)) = t.symbol.split_once('/') {
            if let Ok(price) = t.lastPrice.parse::<f64>() {
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
// KRAKEN
//
#[derive(Deserialize)]
struct KrakenResp {
    result: serde_json::Map<String, serde_json::Value>,
}

pub async fn fetch_kraken(client: &Client) -> Result<Vec<PairPrice>, Box<dyn Error>> {
    let url = "https://api.kraken.com/0/public/Ticker?pair=ALL";
    let resp: KrakenResp = client.get(url).send().await?.json().await?;
    let mut out = Vec::new();

    for (pair, data) in resp.result {
        if let Some(price) = data["c"][0].as_str().and_then(|x| x.parse::<f64>().ok()) {
            // Kraken pairs need cleanup like XBT -> BTC
            let norm_pair = pair.replace("XBT", "BTC");
            if norm_pair.len() >= 6 {
                let (base, quote) = norm_pair.split_at(3);
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
// GATE.IO
//
#[derive(Deserialize)]
struct GateTicker {
    currency_pair: String,
    last: String,
}

pub async fn fetch_gateio(client: &Client) -> Result<Vec<PairPrice>, Box<dyn Error>> {
    let url = "https://api.gate.io/api/v4/spot/tickers";
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
