#[cfg(feature = "gcloud")]
use crate::util::gcloud;
use serde::Deserialize;
#[cfg(feature = "gcloud")]
use tracing::{info, warn};
use tracing_subscriber::filter::{EnvFilter, LevelFilter};
#[cfg(feature = "gcloud")]
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum Format {
    /// log in JSON Format
    Json,
    /// log in pretty, human-readable format
    Human,

    /// google cloud compatible structured logging: <https://cloud.google.com/logging/docs/structured-logging>
    #[cfg(feature = "gcloud")]
    Stackdriver,
}

/// initialize the tracing subscriber.
pub async fn init(format: &Format) {
    let env_filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .from_env_lossy();

    match format {
        Format::Json => tracing_subscriber::fmt()
            .json()
            .with_env_filter(env_filter)
            .init(),
        Format::Human => tracing_subscriber::fmt()
            .pretty()
            .with_env_filter(env_filter)
            .init(),
        #[cfg(feature = "gcloud")]
        Format::Stackdriver => {
            let stackdriver = tracing_stackdriver::layer();
            let registry = tracing_subscriber::registry().with(env_filter);

            // try to fetch the project_id and activate cloud-tracing if it's available
            let project_id = gcloud::fetch_project_id().await;
            match project_id {
                Ok(project_id) => {
                    let stackdriver = stackdriver.with_cloud_trace(
                        tracing_stackdriver::CloudTraceConfiguration {
                            project_id: project_id.to_string(),
                        },
                    );
                    let opentelemetry = tracing_opentelemetry::layer();

                    registry.with(stackdriver).with(opentelemetry).init();

                    info!("cloud-tracing active, project_id: {}", project_id);
                }
                Err(e) => {
                    registry.with(stackdriver).init();

                    warn!(
                        "failed to fetch project_id from Google Cloud, cloud-tracing not active: {}",
                        e
                    );
                }
            }
        }
    }
}
