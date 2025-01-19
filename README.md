# Miffy

My diffy.

A shadow-testing proxy: Send requests to a "*reference*" implementation, send the request to a "
*candidate*"
implementation, always respond with the "*reference*" implementation and log/publish both responses
if they are not
equal.

## Design goals

- *reference* always wins: if the *candidate* fails, is slow, not available or whatever it should
  not impact the
  *reference*, always return a response as fast as possible
- offload complex comparison: only basic comparison, in case of doubt publish both responses as
  different to kafka

## Development

Required tools:

- [rust](https://www.rust-lang.org/tools/install)
- docker-compose
- [kcat](https://github.com/edenhill/kcat)

Native libraries (required for [rdakafka](https://github.com/fede1024/rust-rdkafka?tab=readme-ov-file#installation))

- `libssl-dev`
- `libsasl2-dev`

## Demo

* prepare the config: copy `config.sample.toml` to `config.toml`
* start the demo-servers: `cargo run --example demo`. This will start two servers listening to
  `localhost:3000` (the
  reference) and `localhost:3001` (the candidate) with one endpoint: `/api/{value}`.
* start kafka: `docker-compose up -d`
* start **miffy**: `cargo run`.
* send a request to a path under test: `curl http://localhost:8080/api/3`
* send a request to any other path: `curl http://localhost:8080`
* observe results in kafka: `kcat -b localhost:9092 -e -t miffy`

# Configuration

Miffy looks for a file `config.toml` in it's working-directory. You can point to a specific file via `MIFFY_CONFIG`

See `config.default.toml` for an explanation of different values. These values may be overriden via env-variable prefixed with `MIFFY_`.

Miffy uses *rdkafka* internally and allows to set all its [properties](https://github.com/confluentinc/librdkafka/blob/master/CONFIGURATION.md), in `config.toml` section `[kafka]`.
Since those values typically need to be set via environment variable (e.g. password for kafka, `sasl.password`), they may be set/overriden via `KAFKA_SASL_PASSWORD` etc.

# Benchmarking

To estimate the rough overhead added by miffy, there are [just](https://just.systems/) recipes to
run [wrk](https://github.com/wg/wrk).

* increase the open-file-limit for the shell running the demo: `ulimit -n 8182`
* start the demo-servers: `cargo run --example demo --release`
* start kafka: `docker-compose up -d`
* increase open-file limit for the shell running miffy: `ulimit -n 8182`
* start **miffy** (without access-logging): `RUST_LOG=warn cargo run --release`.
* run the benchmark: `just bench`

# License

Licensed under either of

* Apache License, Version 2.0
  ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license
  ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
