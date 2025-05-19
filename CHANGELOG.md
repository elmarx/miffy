# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.0.1](https://github.com/elmarx/miffy/compare/v1.0.0...v1.0.1) - 2025-05-19

### Fixed

- fix value of "upstream" header in demo
- fix send mutated request (with shadow-test-role-header) to upstream
- *(deps)* update rust crate tower-http to v0.6.4
- *(deps)* update rust crate tokio to v1.45.0
- *(deps)* update rust crate tokio to v1.44.2 [security]
- *(deps)* update rust crate anyhow to v1.0.98
- *(deps)* update rust crate hyper-util to v0.1.11
- *(deps)* update rust crate tracing-opentelemetry to 0.30.0

### Other

- *(deps)* update renovatebot/github-action action to v42.0.3
- *(deps)* update rust docker tag to v1.87.0
- *(deps)* update renovatebot/github-action action to v42
- *(deps)* update rust crate axum to v0.8.4
- update zstd-sys
- *(deps)* update rust docker tag to v1.86.0
- *(deps)* update renovatebot/github-action action to v41.0.22
- *(deps)* update renovatebot/github-action action to v41.0.21
- *(deps)* update renovatebot/github-action action to v41.0.20
- *(deps)* update renovatebot/github-action action to v41.0.19

## [1.0.0](https://github.com/elmarx/miffy/compare/v0.1.0...v1.0.0) - 2025-04-01

### Other

- first stable release: miffy has been used in production now for multiple weeks

## [0.1.0](https://github.com/elmarx/miffy/releases/tag/v0.1.0) - 2025-04-01

### Added

- configurable routes
- traffic-mirroring in separate task
- basic diffing of JSON
- publishing of differences to kafka
- configurable kafka-keys
- header `x-shadow-test-role` indicating role in test