FROM rust:1.84.0 AS builder
ARG REVISION
WORKDIR /usr/src

RUN apt-get update && apt-get install -y libssl-dev libsasl2-dev
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
COPY src src
RUN cargo install --locked --path . --root /usr/local

FROM debian:stable-slim

RUN apt-get update && apt-get install -y libssl3 libsasl2-2
ENV MIFFY_LOG_JSON=1
RUN adduser --system miffy
COPY --from=builder /usr/local/bin/miffy /usr/local/bin

USER miffy
WORKDIR /home/miffy
COPY config.default.toml .

CMD [ "miffy" ]
