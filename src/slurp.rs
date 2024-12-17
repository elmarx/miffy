use http_body_util::{BodyExt, Full};
use hyper::body::Bytes;
use hyper::{Request, Response};

pub async fn slurp_request(
    req: Request<hyper::body::Incoming>,
) -> hyper::Result<Request<Full<Bytes>>> {
    let (parts, body) = req.into_parts();
    let body = body.collect().await?.to_bytes();
    let body = Full::new(body);
    Ok(Request::from_parts(parts, body))
}

pub async fn slurp_response(
    res: Response<hyper::body::Incoming>,
) -> hyper::Result<Response<Bytes>> {
    let (head, body) = res.into_parts();

    let body = body.collect().await.unwrap();

    Ok(Response::from_parts(head, body.to_bytes()))
}
