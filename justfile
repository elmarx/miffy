# run tests/checks that are also run by github actions
ci:
    cargo fmt --all -- --check
    cargo check --tests --examples
    cargo check --all-targets --no-default-features
    cargo nextest run
    cargo clippy -- -D warnings

bench: bench_reference bench_miffy_proxy bench_miffy_mirror

bench_miffy_proxy:
    @echo "miffy proxying"
    wrk -t8 -c400 -d30s http://127.0.0.1:8080/

bench_miffy_mirror:
    @echo "miffy mirroring"
    wrk -t8 -c400 -d30s http://127.0.0.1:8080/api/23

bench_reference:
    @echo "reference"
    wrk -t8 -c400 -d30s http://127.0.0.1:3000/api/23
