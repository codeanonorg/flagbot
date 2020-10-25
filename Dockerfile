FROM rust:1.46-alpine as builder
WORKDIR /opt/src/bot
RUN apk add alpine-sdk openssl-dev
COPY Cargo.toml .
COPY Cargo.lock .
COPY src src
RUN mkdir .cargo && cargo vendor > .cargo/config
RUN cargo build --release

FROM alpine
WORKDIR /opt/bot
COPY --from=builder /opt/src/bot/target/release/flagbot .
CMD ["./flagbot"]