# Stage 1: Build the executable
FROM lukemathwalker/cargo-chef:latest-rust-1 AS chef 
WORKDIR /app 

FROM chef AS builder
COPY . .
RUN cargo chef prepare --recipe-path recipe.json
#builds dependencies - caching!
RUN cargo chef cook --release --recipe-path recipe.json

COPY . .
RUN cargo build --release --bin gateway-api

# Stage 2: Create the runtime image
FROM debian:latest

RUN apt-get update && apt-get install -y libssl-dev ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder app/target/release/gateway-api /usr/local/bin

# Expose the port the axum server runs on - Cloud Run uses 8080
EXPOSE 8080
CMD ["gateway-api"]
