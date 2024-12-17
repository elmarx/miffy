FROM rust:1.83.0 AS builder
ARG REVISION
WORKDIR /usr/src

COPY Cargo.toml Cargo.lock ./
COPY src src
RUN --mount=type=cache,target=/usr/local/cargo/registry --mount=type=cache,target=/usr/src/target REVISION=$REVISION cargo install --locked --path . --root /usr/local

FROM debian:stable-slim

RUN adduser --system miffy
COPY --from=builder /usr/local/bin/miffy /usr/local/bin

USER miffy
WORKDIR /home/miffy

CMD [ "miffy" ]
