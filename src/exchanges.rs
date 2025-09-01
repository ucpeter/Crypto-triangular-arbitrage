use crate::models::MarketPrice;
use serde_json::Value;

pub async fn fetch_binance() -> Vec<MarketPrice> {
    let url = "https://api.binance.com/api/v3/ticker/price";
    let data: Vec<crate::models::BinanceTicker> =
        reqwest::get(url).await.unwrap().json().await.unwrap();
    data.into_iter()
        .filter_map(|t| t.price.parse::<f64>().ok().map(|p| MarketPrice {
            symbol: t.symbol,
            price: p,
            exchange: "Binance".to_string(),
        }))
        .collect()
}

pub async fn fetch_kucoin() -> Vec<MarketPrice> {
    let url = "https://api.kucoin.com/api/v1/market/allTickers";
    let resp: Value = reqwest::get(url).await.unwrap().json().await.unwrap();
    let tickers = resp["data"]["ticker"].as_array().unwrap();
    tickers
        .iter()
        .filter_map(|t| {
            let symbol = t["symbol"].as_str()?.replace("-", "");
            let price = t["last"].as_str()?.parse::<f64>().ok()?;
            Some(MarketPrice {
                symbol,
                price,
                exchange: "KuCoin".to_string(),
            })
        })
        .collect()
}

pub async fn fetch_bybit() -> Vec<MarketPrice> {
    let url = "https://api.bybit.com/v5/market/tickers?category=spot";
    let resp: Value = reqwest::get(url).await.unwrap().json().await.unwrap();
    let tickers = resp["result"]["list"].as_array().unwrap();
    tickers
        .iter()
        .filter_map(|t| {
            let symbol = t["symbol"].as_str()?.to_string();
            let price = t["lastPrice"].as_str()?.parse::<f64>().ok()?;
            Some(MarketPrice {
                symbol,
                price,
                exchange: "Bybit".to_string(),
            })
        })
        .collect()
}

pub async fn fetch_gate() -> Vec<MarketPrice> {
    let url = "https://api.gateio.ws/api/v4/spot/tickers";
    let resp: Value = reqwest::get(url).await.unwrap().json().await.unwrap();
    resp.as_array()
        .unwrap()
        .iter()
        .filter_map(|t| {
            let pair = t["currency_pair"].as_str()?.replace("_", "");
            let price = t["last"].as_str()?.parse::<f64>().ok()?;
            Some(MarketPrice {
                symbol: pair,
                price,
                exchange: "Gate.io".to_string(),
            })
        })
        .collect()
}

pub async fn fetch_kraken() -> Vec<MarketPrice> {
    let url = "https://api.kraken.com/0/public/Ticker?pair=BTCUSDT,ETHUSDT";
    let resp: Value = reqwest::get(url).await.unwrap().json().await.unwrap();
    resp["result"]
        .as_object()
        .unwrap()
        .iter()
        .filter_map(|(pair, data)| {
            let price = data["c"][0].as_str()?.parse::<f64>().ok()?;
            Some(MarketPrice {
                symbol: pair.to_string(),
                price,
                exchange: "Kraken".to_string(),
            })
        })
        .collect()
        }
