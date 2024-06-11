# Stage 1: Build the executable
FROM rust:1.75 as builder
WORKDIR /usr/src/gateway-api
COPY . .

# Set the CARGO_HOME environment variable to include the spvm-rs dependency
ENV CARGO_HOME=/usr/packages/spvm-rs

RUN cargo build --package gateway-api --release

# Stage 2: Create the runtime image
FROM debian:latest

RUN apt-get update && apt-get install -y libssl-dev ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/src/gateway-api/target/release/gateway-api /usr/local/bin/gateway-api

# Expose the port the axum server runs on - Cloud Run uses 8080
EXPOSE 8080
CMD ["gateway-api"]
