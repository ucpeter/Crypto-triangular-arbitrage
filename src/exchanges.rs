use crate::models::PairPrice;
use reqwest::Client;

/// Fetch normalized spot prices from supported exchanges
pub async fn fetch_exchange_data(exchange: &str) -> Result<Vec<PairPrice>, String> {
    let client = Client::new();

    match exchange.to_lowercase().as_str() {
        "binance" => {
            let url = "https://api.binance.com/api/v3/ticker/price";
            let resp = client.get(url).send().await.map_err(|e| e.to_string())?;
            let data: Vec<serde_json::Value> = resp.json().await.map_err(|e| e.to_string())?;

            let mut prices = Vec::new();
            for item in data {
                if let (Some(symbol), Some(price_str)) = (item["symbol"].as_str(), item["price"].as_str()) {
                    if let Ok(price) = price_str.parse::<f64>() {
                        // crude split base/quote (Binance symbols like BTCUSDT, ETHBTC, etc.)
                        let (base, quote) = split_symbol(symbol);
                        prices.push(PairPrice {
                            base,
                            quote,
                            price,
                            is_spot: true,
                        });
                    }
                }
            }
            Ok(prices)
        }

        "kucoin" => {
            let url = "https://api.kucoin.com/api/v1/market/allTickers";
            let resp = client.get(url).send().await.map_err(|e| e.to_string())?;
            let data: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;

            let mut prices = Vec::new();
            if let Some(tickers) = data["data"]["ticker"].as_array() {
                for item in tickers {
                    if let (Some(symbol), Some(price_str)) = (item["symbol"].as_str(), item["last"].as_str()) {
                        if let Ok(price) = price_str.parse::<f64>() {
                            if let Some((base, quote)) = symbol.split_once('-') {
                                prices.push(PairPrice {
                                    base: base.to_string(),
                                    quote: quote.to_string(),
                                    price,
                                    is_spot: true,
                                });
                            }
                        }
                    }
                }
            }
            Ok(prices)
        }

        "kraken" => {
            let url = "https://api.kraken.com/0/public/Ticker?pair=BTCUSD,ETHUSD,ETHBTC";
            let resp = client.get(url).send().await.map_err(|e| e.to_string())?;
            let data: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;

            let mut prices = Vec::new();
            if let Some(result) = data["result"].as_object() {
                for (pair, info) in result {
                    if let Some(price_str) = info["c"][0].as_str() {
                        if let Ok(price) = price_str.parse::<f64>() {
                            // Kraken symbols are funky, e.g., XXBTZUSD → BTC/USD
                            let (base, quote) = normalize_kraken_symbol(pair);
                            prices.push(PairPrice {
                                base,
                                quote,
                                price,
                                is_spot: true,
                            });
                        }
                    }
                }
            }
            Ok(prices)
        }

        "bybit" => {
            let url = "https://api.bybit.com/v5/market/tickers?category=spot";
            let resp = client.get(url).send().await.map_err(|e| e.to_string())?;
            let data: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;

            let mut prices = Vec::new();
            if let Some(list) = data["result"]["list"].as_array() {
                for item in list {
                    if let (Some(symbol), Some(price_str)) = (item["symbol"].as_str(), item["lastPrice"].as_str()) {
                        if let Ok(price) = price_str.parse::<f64>() {
                            if let Some((base, quote)) = split_symbol_bybit(symbol) {
                                prices.push(PairPrice {
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
            Ok(prices)
        }

        "gate" | "gateio" => {
            let url = "https://api.gateio.ws/api/v4/spot/tickers";
            let resp = client.get(url).send().await.map_err(|e| e.to_string())?;
            let data: Vec<serde_json::Value> = resp.json().await.map_err(|e| e.to_string())?;

            let mut prices = Vec::new();
            for item in data {
                if let (Some(symbol), Some(price_str)) = (item["currency_pair"].as_str(), item["last"].as_str()) {
                    if let Ok(price) = price_str.parse::<f64>() {
                        if let Some((base, quote)) = symbol.split_once('_') {
                            prices.push(PairPrice {
                                base: base.to_uppercase(),
                                quote: quote.to_uppercase(),
                                price,
                                is_spot: true,
                            });
                        }
                    }
                }
            }
            Ok(prices)
        }

        _ => Err(format!("Exchange {} not supported", exchange)),
    }
}

/// Split Binance-style symbols (BTCUSDT → BTC, USDT)
fn split_symbol(symbol: &str) -> (String, String) {
    let quotes = ["USDT", "BTC", "ETH", "BUSD", "USDC", "EUR", "USD"];
    for q in quotes {
        if symbol.ends_with(q) {
            let base = symbol.trim_end_matches(q).to_string();
            return (base, q.to_string());
        }
    }
    (symbol.to_string(), "".to_string())
}

/// Split Bybit symbols (e.g., ETHUSDT → ETH/USDT)
fn split_symbol_bybit(symbol: &str) -> Option<(String, String)> {
    let quotes = ["USDT", "USDC", "BTC", "ETH"];
    for q in quotes {
        if symbol.ends_with(q) {
            let base = symbol.trim_end_matches(q).to_string();
            return Some((base, q.to_string()));
        }
    }
    None
}

/// Normalize Kraken symbols to readable base/quote
fn normalize_kraken_symbol(raw: &str) -> (String, String) {
    let mut base = raw.chars().take(3).collect::<String>();
    let mut quote = raw.chars().skip(3).collect::<String>();

    base = base.replace("XBT", "BTC").replace("XETH", "ETH");
    quote = quote.replace("ZUSD", "USD").replace("ZEUR", "EUR");

    (base, quote)
                        }
