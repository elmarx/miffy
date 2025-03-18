use crate::http::slurp;
use bytes::Bytes;
use http::Request;
use http_body_util::Empty;
use hyper_util::client::legacy::Client;
use hyper_util::client::legacy::connect::HttpConnector;

/// fetch the Google cloud project-id
/// see <https://cloud.google.com/compute/docs/metadata/predefined-metadata-keys>
pub async fn fetch_project_id() -> anyhow::Result<String> {
    let uri = "http://metadata.google.internal/computeMetadata/v1/project/project-id"
        .parse::<hyper::Uri>()
        .expect("hardcoded URI must be valid");

    let client = Client::builder(hyper_util::rt::TokioExecutor::new()).build(HttpConnector::new());

    let req = Request::builder()
        .uri(uri)
        .header("Metadata-Flavor", "Google")
        .body(Empty::<Bytes>::new())?;

    let response = client.request(req).await?;
    let response = slurp::response(response).await?;

    let project_id = String::from_utf8(response.body().to_vec())?;

    Ok(project_id)
}
