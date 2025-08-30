# Stage 1: Build Rust binary
FROM rust:1.82 as builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release

# Stage 2: Python for Streamlit UI
FROM python:3.11-slim
WORKDIR /app
COPY --from=builder /app/target/release/triangular_arbitrage /app/triangular_arbitrage
COPY ui ./ui
COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt
EXPOSE 8501
CMD ["streamlit", "run", "ui/ui.py", "--server.port=8501", "--server.address=0.0.0.0"]
