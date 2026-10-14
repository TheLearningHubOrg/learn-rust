# Build Stage
FROM rust:1.80-alpine AS builder
WORKDIR /app
RUN apk add --no-cache musl-dev
COPY Cargo.toml ./
COPY src ./src
RUN cargo build --release

# Production Stage
FROM alpine:3.20
WORKDIR /app
COPY --from=builder /app/target/release/learn-rust /app/server
EXPOSE 8080
CMD ["/app/server"]
