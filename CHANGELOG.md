# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0](https://github.com/elmarx/miffy/releases/tag/v0.1.0) - 2025-04-01

### Fixed

- fix some pedantic clippys
- fix some suggestions by clippy
- fix image caching

### Other

- init release-plz
- add description
- attach headers to requests to indicate wether a service takes part in shadow-testing and which role the service plays
- Update renovatebot/github-action action to v41.0.18
- Update rust Docker tag to v1.85.1
- Update renovatebot/github-action action to v41.0.17
- fetch project-id from google metadata service
- make gcloud-support/stackdriver optional
- support stackdriver logging
- make logging-format an enum
- add (default) just-recipe to execute tests/checks
- Update renovatebot/github-action action to v41.0.16
- execute body-parsing outside of the "main" task
- update dependencies
- update dependencies
- do not cleanup/delete old images
- do not copy the default-config into the dockerfile
- explicitly install/setup rust toolchain
- update dependencies
- upgrade to edition 2024, rust version 1.85
- Update renovatebot/github-action action to v41.0.14
- parse body as JSON for requests (i.e.: look at the accept header, too)
- update dependencies
- put routing-information into the request
- make kafka-key configurable and put route-parameters into the kafka-message
- read and use config.default.toml for dry and consistent defaults
- implement management-port and provide health-check-endpoint
- set a route's path as key when publishing to kafka
- update dependencies
- Update renovatebot/github-action action to v41.0.13
- increase the number of versions to keep
- use continuos, semantic, renovate-compatible docker-tags
- update dependencies
- Update rust Docker tag to v1.84.1
- run renovate when changing/updating dependencies
- Update renovatebot/github-action action to v41.0.12
- Update renovatebot/github-action action to v41.0.11
- switch to nextest for test-execution
- set explicit rust-version
- update dependencies
- set a proper semver-tag for the docker-image
- tag the latest-image as latest
- extend json-detection to detect proprietary/extended json formats, too
- Update renovatebot/github-action action to v41.0.10
- enable to override kafka-properties via env-variables
- pass rdkafka-properties from config directly to rdkafka
- enable sasl/ssl for kafka
- update dependencies
- Update renovatebot/github-action action to v41.0.9
- Update rust Docker tag to v1.84.0
- polish configuration
- Update renovatebot/github-action action to v41.0.8
- update dependencies
- add license to project
- use tokio/tcp-keep-alive in http-client
- add simple benchmark-commands
- enable log-level configuration via RUST_LOG
- lower renovate interval to daily
- Update renovatebot/github-action action to v41.0.7
- basic diffing of responses, to only publish differences
- record actual url used for the candidate/reference
- configure per-path routes
- introduce basic config
- use old school layer caching instead of using docker cache
- modularize service
- refine a clear domain-model
- modularize app
- update crates
- publish request-failures to kafka, too
- put request-code into function
- early handle downstream errors
- simplify according to clippy suggestion
- configure access-logging
- setup logging
- proper error handling - no more unwrap
- add basic dockerfile
- add gha
- send test-samples as json via kafka
- introduce struct to examine sample-requests
- setup renovate
- add documentation for required tools
- add initial kafka support
- add basic documentation
- execute shadow-testing only for selected routes/paths
- split handler
- move handler into struct
- use jemalloc
- basic hyper-based poc sending request to candidate and reference
- run two servers in demo
- buffer/fetch response before streaming it to the client
- initial reverse proxy
- initial commit
