# Miffy

My diffy.

A shadow-testing proxy: Send requests to a "*reference*" implementation, send the request to a "*candidate*"
implementation, always respond with the "*reference*" implementation and log/publish both responses if they are not
equal.

## Design goals

- *reference* always wins: if the *candidate* fails, is slow, not available or whatever it should not impact the
  *reference*, always return a response as fast as possible
- offload complex comparison: only basic comparison, in case of doubt publish both responses as different to kafka

## Demo

* start the demo-servers: `cargo run --example demo`. This will start two servers listening to `localhost:3000` (the
  reference) and `localhost:3001` (the candidate) with one endpoint: `/api/{value}`.
* start kafka: `docker-compose up -d`
* start **miffy**: `cargo run`.
* send a request to a path under test: `curl http://localhost:8080/api/3`
* send a request to any other path: `curl http://localhost:8080`
* observe results in kafka: `kcat -b localhost:9092 -e -t miffy`