FROM rust:1.94.1-slim AS builder

WORKDIR /app

COPY rust-server/.sqlx ./.sqlx
COPY rust-server/Cargo.toml rust-server/Cargo.lock ./

RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -f target/release/deps/rust_server*

COPY rust-server/src ./src

RUN cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/lists/*

RUN useradd -r -s /bin/false appuser

COPY --from=builder /app/target/release/rust-server ./

USER appuser

EXPOSE 3000

CMD [ "./rust-server" ]
