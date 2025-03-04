use http::Response;
use std::time::Duration;
use tower_http::trace::{DefaultOnRequest, HttpMakeClassifier, MakeSpan, OnResponse, TraceLayer};
use tracing::Span;

pub fn new_trace_layer<B, R>() -> TraceLayer<
    HttpMakeClassifier,
    impl MakeSpan<B> + Clone,
    DefaultOnRequest,
    impl OnResponse<R> + Clone,
> {
    TraceLayer::new_for_http()
        .make_span_with(|request: &http::Request<B>| {
            tracing::info_span!("request", "http_request.request_method" = %request.method(), "http_request.request_url" = %request.uri(), "http_request.version" = ?request.version(), "log_type" = "access")
        })
        .on_response(|response: &Response<R>, latency: Duration, span: &Span| {
            let latency = latency.as_millis();
            let latency = format!("{}ms", latency);

            let request = span.field("http_request.request_method").map(|f| f.);

            tracing::info!("http_request.latency" = %latency, "http_request.status" = response.status().as_u16(), "http_request.request_method" = request);
        })
}
