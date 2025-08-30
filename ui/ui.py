import streamlit as st
import subprocess
import json
import pandas as pd

st.set_page_config(page_title="Rust Arbitrage Scanner", layout="wide")
st.title("🚀 Rust-Powered Triangular Arbitrage Scanner")

if st.button("Run Scan"):
    with st.spinner("Running Rust Scanner..."):
        try:
            result = subprocess.run(
                ["../target/release/triangular_arbitrage"],
                stdout=subprocess.PIPE,
                text=True,
                check=True
            )
            data = json.loads(result.stdout)
            if data:
                df = pd.DataFrame(data)
                df = df.sort_values(by="profit_percent", ascending=False)
                st.dataframe(df)
            else:
                st.warning("No profitable opportunities found.")
        except Exception as e:
            st.error(f"Error running scanner: {e}")
