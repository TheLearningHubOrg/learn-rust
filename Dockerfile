# Learn Rust starter service: axum on Tokio, built in release mode.

# ---- build: cache dependencies separately from source -----------------------
FROM rust:1-slim-bookworm AS build
WORKDIR /src
COPY Cargo.toml Cargo.lock ./
# Compile dependencies against a dummy main so they cache between source edits.
RUN mkdir src && echo "fn main() {}" > src/main.rs \
 && cargo build --release --locked \
 && rm -rf src
COPY . .
# touch so cargo sees the real sources as newer than the cached dummy build
RUN touch src/main.rs src/lib.rs && cargo build --release --locked

# ---- runtime ----------------------------------------------------------------
FROM debian:bookworm-slim AS runtime
RUN apt-get update \
 && apt-get install -y --no-install-recommends ca-certificates \
 && rm -rf /var/lib/apt/lists/* \
 && useradd --uid 10001 --no-create-home app
COPY --from=build /src/target/release/learn-rust /usr/local/bin/learn-rust
ENV PORT=8080
EXPOSE 8080
USER app
CMD ["learn-rust"]
