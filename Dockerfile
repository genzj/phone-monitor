# syntax=docker/dockerfile:1

# Build stage
FROM rust:1.84.0 AS build
WORKDIR /app
COPY . .
RUN cargo build --release

# Run stage
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates openssl && rm -rf /var/lib/apt/lists/*
COPY --from=build /app/target/release/phone-monitor /usr/local/bin/phone-monitor

CMD ["phone-monitor"]
