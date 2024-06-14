# Stage 1: Build the executable
FROM rust:1.75 as builder
#WORKDIR /usr/src/
COPY . .

RUN cargo build --release

# Stage 2: Create the runtime image
FROM debian:latest

RUN apt-get update && apt-get install -y libssl-dev ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/src/target/release/gateway-api /usr/local/bin/gateway-api

# Expose the port the axum server runs on - Cloud Run uses 8080
EXPOSE 8080
CMD ["gateway-api"]
