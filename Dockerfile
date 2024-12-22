FROM rust:1.83.0 AS builder
ARG REVISION
WORKDIR /usr/src

COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
COPY src src
RUN cargo install --locked --path . --root /usr/local

FROM debian:stable-slim

ENV LOG_JSON=1
RUN adduser --system miffy
COPY --from=builder /usr/local/bin/miffy /usr/local/bin

USER miffy
WORKDIR /home/miffy
COPY config.default.toml .

CMD [ "miffy" ]
