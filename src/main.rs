use http_body_util::BodyExt;
use http_body_util::Full;
use hyper::body::Bytes;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response};
use hyper::{StatusCode, Uri};
use hyper_util::client::legacy::connect::HttpConnector;
use hyper_util::client::legacy::Client;
use hyper_util::rt::TokioIo;
use std::convert::Infallible;
use tokio::net::TcpListener;
use tokio::sync::oneshot;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let listener = TcpListener::bind("0.0.0.0:8080").await?;

    // We start a loop to continuously accept incoming connections
    loop {
        let (stream, _) = listener.accept().await?;

        // Use an adapter to access something implementing `tokio::io` traits as if they implement
        // `hyper::rt` IO traits.
        let io = TokioIo::new(stream);

        // Spawn a tokio task to serve multiple connections concurrently
        tokio::task::spawn(async move {
            // Finally, we bind the incoming connection to our `hello` service
            if let Err(err) = http1::Builder::new()
                // `service_fn` converts our function in a `Service`
                .serve_connection(io, service_fn(handler))
                .await
            {
                eprintln!("Error serving connection: {:?}", err);
            }
        });
    }
}

async fn slurp_request(req: Request<hyper::body::Incoming>) -> hyper::Result<Request<Full<Bytes>>> {
    let (parts, body) = req.into_parts();
    let body = body.collect().await?.to_bytes();
    let body = Full::new(body);
    Ok(Request::from_parts(parts, body))
}

async fn slurp_response(res: Response<hyper::body::Incoming>) -> hyper::Result<Response<Bytes>> {
    let (head, body) = res.into_parts();

    let body = body.collect().await.unwrap();

    Ok(Response::from_parts(head, body.to_bytes()))
}

async fn handler(req: Request<hyper::body::Incoming>) -> Result<Response<Full<Bytes>>, Infallible> {
    let mut reference_request = slurp_request(req).await.unwrap();
    let reference_client =
        Client::builder(hyper_util::rt::TokioExecutor::new()).build(HttpConnector::new());

    let candidate_client = reference_client.clone();

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

    tokio::spawn(async move {
        *candidate_request.uri_mut() = Uri::try_from(candidate_uri).unwrap();

        let candidate_response = candidate_client
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

    let reference_response = reference_client
        .request(reference_request)
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)
        .unwrap();

    let response = slurp_response(reference_response).await.unwrap();

    // send the reference-respons over to the candidate-task
    tx.send(response.clone()).unwrap();

    Ok(response.map(Full::new))
}
