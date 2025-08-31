# Stage 1: Build Rust binary
FROM rust:1.82 as builder
WORKDIR /app

# Copy Cargo.toml first to generate Cargo.lock if missing
COPY Cargo.toml ./
RUN cargo generate-lockfile

# Copy the rest of the Rust source code
COPY src ./src

# Build Rust binary
RUN cargo build --release

# Stage 2: Python for Streamlit UI
FROM python:3.11-slim
WORKDIR /app

# Copy compiled Rust binary
COPY --from=builder /app/target/release/triangular_arbitrage /app/triangular_arbitrage

# Copy UI files and requirements
COPY ui ./ui
COPY requirements.txt .

# Install Python dependencies
RUN pip install --no-cache-dir -r requirements.txt

# Expose Streamlit port
EXPOSE 8501

# Start Streamlit server
CMD ["streamlit", "run", "ui/ui.py", "--server.port=8501", "--server.address=0.0.0.0"]
