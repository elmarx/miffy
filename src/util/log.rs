/// initialize the tracing subscriber.
///
/// by default, it uses human-readable logs, unless the env-variable `LOG_JSON` is set to a `truethy` value, then it emits json-formatted-logs
pub fn init(log_json: bool) {
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
