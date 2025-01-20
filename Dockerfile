# syntax=docker/dockerfile:1

# Build stage
FROM rust:1.84.0 AS build
WORKDIR /app
RUN --mount=type=cache,target=/var/cache/apt,sharing=locked \
    --mount=type=cache,target=/var/lib/apt,sharing=locked \
    apt-get update && \
    apt-get install -y sccache apt-utils
ENV RUSTC_WRAPPER=sccache SCCACHE_DIRECT=true
COPY . .
RUN --mount=type=cache,target=/root/.cargo/registry,sharing=locked \
    --mount=type=cache,target=/root/.cache/sccache,sharing=locked \
    cargo build --release && \
    sccache --show-stats

# Run stage
FROM debian:bookworm-slim
RUN --mount=type=cache,target=/var/cache/apt,sharing=locked \
    --mount=type=cache,target=/var/lib/apt,sharing=locked \
    apt-get update && \
    apt-get install -y ca-certificates openssl && \
    rm -rf /var/lib/apt/lists/*
COPY --from=build /app/target/release/phone-monitor /usr/local/bin/phone-monitor

CMD ["phone-monitor"]
