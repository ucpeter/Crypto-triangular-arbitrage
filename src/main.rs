mod models;
mod utils;
mod exchanges;
mod scanner;

use exchanges::{fetch_markets, fetch_tickers};
use scanner::{build_triangles, evaluate_triangles};
use serde_json::json;

#[tokio::main]
async fn main() {
    let exchanges = vec!["binance", "kucoin", "bybit", "gateio"];
    let mut all_results = vec![];

    for ex in exchanges {
        if let Ok(markets) = fetch_markets(ex).await {
            if let Ok(tickers) = fetch_tickers(ex).await {
                let triangles = build_triangles(&markets);
                let results = evaluate_triangles(ex, &triangles, &tickers);
                all_results.extend(results);
            }
        }
    }

    println!("{}", json!(all_results));
}
