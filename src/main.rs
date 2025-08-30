mod models;
mod utils;
mod exchanges;
mod scanner;

use exchanges::{fetch_markets, fetch_tickers};
use scanner::{build_triangles, evaluate_triangles};
use serde_json;
use std::env;

#[tokio::main]
async fn main() {
    // Capture CLI args (after the binary name)
    let args: Vec<String> = env::args().skip(1).collect();

    // Default to binance + kucoin if no exchanges provided
    let exchanges = if args.is_empty() {
        vec!["binance".to_string(), "kucoin".to_string()]
    } else {
        args
    };

    let mut all_results = vec![];

    // Iterate through selected exchanges
    for ex in &exchanges {
        match fetch_markets(ex).await {
            Ok(markets) => match fetch_tickers(ex).await {
                Ok(tickers) => {
                    let triangles = build_triangles(&markets);
                    let results = evaluate_triangles(ex, &triangles, &tickers);
                    all_results.extend(results);
                }
                Err(e) => eprintln!("Error fetching tickers for {}: {}", ex, e),
            },
            Err(e) => eprintln!("Error fetching markets for {}: {}", ex, e),
        }
    }

    // Output results as JSON for Streamlit to parse
    println!("{}", serde_json::to_string(&all_results).unwrap());
            }
