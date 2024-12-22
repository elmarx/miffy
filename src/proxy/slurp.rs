use crate::proxy::error::Upstream;
use crate::proxy::error::Upstream::ReadBody;
use http_body_util::BodyExt;
use hyper::body::Bytes;
use hyper::{Request, Response};

pub async fn request(req: Request<hyper::body::Incoming>) -> hyper::Result<Request<Bytes>> {
    let (parts, body) = req.into_parts();
    let body = body.collect().await?.to_bytes();
    Ok(Request::from_parts(parts, body))
}

pub async fn response(res: Response<hyper::body::Incoming>) -> Result<Response<Bytes>, Upstream> {
    let (head, body) = res.into_parts();
    let body = body.collect().await.map_err(ReadBody)?;
    Ok(Response::from_parts(head, body.to_bytes()))
}
