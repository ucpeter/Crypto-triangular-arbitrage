import streamlit as st
import subprocess
import json
import pandas as pd

# Page config
st.set_page_config(page_title="Rust Arbitrage Scanner", layout="wide")
st.title("🚀 Rust-Powered Triangular Arbitrage Scanner")

# --- Control Panel ---
with st.expander("⚙️ Scanner Settings", expanded=True):
    st.write("Configure your scan options below:")
    
    # Exchange selection
    all_exchanges = ["binance", "kucoin", "bybit", "gateio"]
    selected_exchanges = st.multiselect(
        "Select up to 5 exchanges", 
        all_exchanges, 
        default=["binance", "kucoin"]
    )

    # Profit filter
    min_profit = st.number_input("Minimum profit %", min_value=0.0, value=0.5, step=0.1)
    max_profit = st.number_input("Maximum profit %", min_value=0.0, value=10.0, step=0.5)

    # Run button
    run_scan = st.button("🔍 Scan Now")

# --- Scan Logic ---
if run_scan:
    if not selected_exchanges:
        st.error("Please select at least one exchange.")
    else:
        with st.spinner("Running Rust scanner..."):
            try:
                # Call Rust binary with exchanges passed as args
                result = subprocess.run(
                    ["../target/release/triangular_arbitrage"] + selected_exchanges,
                    stdout=subprocess.PIPE,
                    text=True,
                    check=True
                )

                data = json.loads(result.stdout)

                # Filter by profit margin
                filtered = [
                    row for row in data
                    if min_profit <= row["profit_percent"] <= max_profit
                ]

                if filtered:
                    df = pd.DataFrame(filtered)
                    df = df.sort_values(by="profit_percent", ascending=False).reset_index(drop=True)
                    st.dataframe(df, use_container_width=True)
                else:
                    st.warning("No opportunities found within your profit range.")
            except Exception as e:
                st.error(f"Error running scanner: {e}")
