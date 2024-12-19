use lenient_bool::LenientBool;
use tower_http::trace::{DefaultMakeSpan, DefaultOnResponse, HttpMakeClassifier, TraceLayer};
use tracing::Level;

pub fn init() {
    let log_json = std::env::var("LOG_JSON").map(|s| s.parse::<LenientBool>().unwrap_or_default())
        == Ok(LenientBool(true));

    if log_json {
        tracing_subscriber::fmt()
            .json()
            .with_max_level(tracing::Level::INFO)
            .init();
    } else {
        tracing_subscriber::fmt()
            .pretty()
            .with_max_level(tracing::Level::INFO)
            .init();
    };
}

pub fn new_trace_layer() -> TraceLayer<HttpMakeClassifier> {
    TraceLayer::new_for_http()
        .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
        .on_response(DefaultOnResponse::new().level(Level::INFO))
}
