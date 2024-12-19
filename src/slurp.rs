use crate::error::Miffy::{ReadRequestBody, ReadResponseBody};
use crate::error::Result;
use http_body_util::BodyExt;
use hyper::body::Bytes;
use hyper::{Request, Response};

pub async fn slurp_request(req: Request<hyper::body::Incoming>) -> Result<Request<Bytes>> {
    let (parts, body) = req.into_parts();
    let body = body.collect().await.map_err(ReadRequestBody)?.to_bytes();
    Ok(Request::from_parts(parts, body))
}

pub async fn slurp_response(res: Response<hyper::body::Incoming>) -> Result<Response<Bytes>> {
    let (head, body) = res.into_parts();
    let body = body.collect().await.map_err(ReadResponseBody)?;
    Ok(Response::from_parts(head, body.to_bytes()))
}
