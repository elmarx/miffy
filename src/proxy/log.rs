use tower_http::trace::{DefaultMakeSpan, DefaultOnResponse, HttpMakeClassifier, TraceLayer};
use tracing::Level;

pub fn new_trace_layer() -> TraceLayer<HttpMakeClassifier> {
    TraceLayer::new_for_http()
        .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
        .on_response(DefaultOnResponse::new().level(Level::INFO))
}
