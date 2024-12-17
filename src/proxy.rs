use crate::slurp::{slurp_request, slurp_response};
use http_body_util::Full;
use hyper::body::Bytes;
use hyper::{Request, Response, StatusCode, Uri};
use hyper_util::client::legacy::connect::HttpConnector;
use hyper_util::client::legacy::Client;
use tokio::sync::oneshot;

#[derive(Clone)]
pub struct Proxy {
    client: Client<HttpConnector, Full<Bytes>>,
}

impl Proxy {
    pub(crate) fn new() -> Self {
        Self {
            client: Client::builder(hyper_util::rt::TokioExecutor::new())
                .build(HttpConnector::new()),
        }
    }

    pub async fn handle(
        &self,
        req: Request<hyper::body::Incoming>,
    ) -> hyper::Result<Response<Full<Bytes>>> {
        let mut reference_request = slurp_request(req).await.unwrap();

        let path = reference_request.uri().path();
        let path_query = reference_request
            .uri()
            .path_and_query()
            .map(|v| v.as_str())
            .unwrap_or(path);

        let candidate_uri = format!("http://127.0.0.1:3001{}", path_query);
        let reference_uri = format!("http://127.0.0.1:3000{}", path_query);

        *reference_request.uri_mut() = Uri::try_from(reference_uri).unwrap();
        let mut candidate_request = reference_request.clone();

        let (tx, rx) = oneshot::channel::<Response<Bytes>>();

        let client = self.client.clone();

        tokio::spawn(async move {
            *candidate_request.uri_mut() = Uri::try_from(candidate_uri).unwrap();

            let candidate_response = client
                .request(candidate_request)
                .await
                .map_err(|_| StatusCode::BAD_GATEWAY)
                .unwrap();

            let response = slurp_response(candidate_response).await.unwrap();

            let candidate_body = response.into_body();
            let reference_response = rx.await.unwrap();
            println!("Candidate: {:#?}", candidate_body);
            println!("Reference: {:#?}", reference_response.body());
        });

        let reference_response = self
            .client
            .request(reference_request)
            .await
            .map_err(|_| StatusCode::BAD_GATEWAY)
            .unwrap();

        let response = slurp_response(reference_response).await.unwrap();

        // send the reference-response over to the candidate-task
        tx.send(response.clone()).unwrap();

        Ok(response.map(Full::new))
    }
}
