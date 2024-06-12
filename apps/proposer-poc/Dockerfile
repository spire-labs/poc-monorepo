# Stage 1: Build the executable
FROM rust:1.75 as builder
WORKDIR /usr/src/proposer-poc

# Install git and openssh-client
RUN apt-get update && apt-get install -y git openssh-client

# Copy the SSH key and set up SSH
COPY id_rsa /root/.ssh/id_rsa
RUN chmod 600 /root/.ssh/id_rsa
RUN ssh-keyscan github.com >> /root/.ssh/known_hosts

RUN git clone git@github.com:spire-labs/spvm-rs.git /usr/src/spvm-rs

COPY . .

# Adjust the path to include the spvm-rs dependency
# RUN mkdir -p /usr/src/spvm-rs/entity
# COPY /usr/src/spvm-rs/entity /usr/src/spvm-rs/entity

# Set the CARGO_HOME environment variable to include the spvm-rs dependency
ENV CARGO_HOME=/usr/src/spvm-rs

RUN cargo build --release

# Stage 2: Create the runtime image
FROM debian:latest

RUN apt-get update && apt-get install -y libssl-dev ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/src/proposer-poc/target/release/proposer-poc /usr/local/bin/proposer-poc

# Expose the port the axum server runs on - Cloud Run uses 8080
EXPOSE 8080
CMD ["proposer-poc"]

