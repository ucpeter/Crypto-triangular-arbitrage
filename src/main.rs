mod exchanges;
mod logic;
mod models;

use crate::exchanges::*;
use crate::logic::find_triangular_arbitrage;
use crate::models::ArbitrageResult;

use eframe::{egui, epi};
use std::sync::{Arc, Mutex};
use tokio::runtime::Runtime;

struct AppState {
    results: Arc<Mutex<Vec<ArbitrageResult>>>,
    rt: Runtime,
    selected_exchanges: Vec<String>,
    min_profit: f64,
}

impl epi::App for AppState {
    fn name(&self) -> &str {
        "Rust Triangular Arbitrage Scanner"
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &epi::Frame) {
        egui::SidePanel::left("control_panel").show(ctx, |ui| {
            ui.heading("Control Panel");

            ui.label("Select Exchanges:");
            let all_exchanges = vec![
                "Binance", "KuCoin", "Bybit", "Gate.io", "Kraken",
            ];
            for exchange in all_exchanges {
                let mut selected = self.selected_exchanges.contains(&exchange.to_string());
                if ui.checkbox(&mut selected, exchange).changed() {
                    if selected {
                        self.selected_exchanges.push(exchange.to_string());
                    } else {
                        self.selected_exchanges.retain(|x| x != exchange);
                    }
                }
            }

            ui.separator();
            ui.label("Minimum Profit %:");
            ui.add(egui::DragValue::new(&mut self.min_profit).speed(0.1).suffix("%"));

            if ui.button("Scan Now").clicked() {
                let exchanges = self.selected_exchanges.clone();
                let results_arc = self.results.clone();
                let min_profit = self.min_profit;
                self.rt.spawn(async move {
                    let mut all_prices = vec![];
                    if exchanges.contains(&"Binance".to_string()) {
                        all_prices.extend(fetch_binance().await);
                    }
                    if exchanges.contains(&"KuCoin".to_string()) {
                        all_prices.extend(fetch_kucoin().await);
                    }
                    if exchanges.contains(&"Bybit".to_string()) {
                        all_prices.extend(fetch_bybit().await);
                    }
                    if exchanges.contains(&"Gate.io".to_string()) {
                        all_prices.extend(fetch_gate().await);
                    }
                    if exchanges.contains(&"Kraken".to_string()) {
                        all_prices.extend(fetch_kraken().await);
                    }

                    let mut results = find_triangular_arbitrage(all_prices);
                    results.retain(|r| r.profit_after >= min_profit);

                    let mut locked = results_arc.lock().unwrap();
                    *locked = results;
                });
            }
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Arbitrage Results");
            let results = self.results.lock().unwrap();
            if results.is_empty() {
                ui.label("No opportunities yet. Click 'Scan Now'.");
            } else {
                egui::Grid::new("results_table")
                    .striped(true)
                    .show(ui, |ui| {
                        ui.label("Route");
                        ui.label("Profit Before (%)");
                        ui.label("Fee (%)");
                        ui.label("Profit After (%)");
                        ui.label("Spread");
                        ui.end_row();

                        for res in results.iter() {
                            ui.label(&res.route);
                            ui.label(format!("{:.2}", res.profit_before));
                            ui.label(format!("{:.2}", res.fee));
                            ui.label(format!("{:.2}", res.profit_after));
                            ui.label(format!("{:.2}", res.spread));
                            ui.end_row();
                        }
                    });
            }
        });
    }
}

fn main() -> eframe::Result<()> {
    let rt = Runtime::new().unwrap();
    let app = AppState {
        results: Arc::new(Mutex::new(vec![])),
        rt,
        selected_exchanges: vec!["Binance".to_string()],
        min_profit: 1.0,
    };
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(Box::new(app), native_options)
                                                                    }
