use crate::http::slurp;
use bytes::Bytes;
use http::{Request, Response, Uri};
use http_body_util::Full;
use hyper_util::client::legacy::connect::HttpConnector;

pub type Client = hyper_util::client::legacy::Client<HttpConnector, Full<Bytes>>;

pub trait UpstreamExt {
    async fn upstream(
        &self,
        req: http::Request<Bytes>,
        uri: &str,
    ) -> Result<Response<Bytes>, crate::http::error::Upstream>;
}

impl UpstreamExt for Client {
    async fn upstream(
        &self,
        mut req: Request<Bytes>,
        uri: &str,
    ) -> Result<Response<Bytes>, crate::http::error::Upstream> {
        *req.uri_mut() = Uri::try_from(uri)?;

        let response = self
            .request(req.map(Full::new))
            .await
            .map_err(crate::http::error::Upstream::Request)?;

        slurp::response(response).await
    }
}
