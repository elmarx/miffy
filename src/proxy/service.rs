use crate::diff::dispatcher::Dispatcher;
use crate::diff::mirror::Mirror;
use crate::diff::tx_ext::TxExt;
use crate::http::client::{Client, UpstreamExt};
use crate::http::model::RequestMode;
use crate::http::{SHADOW_TEST_HEADER, error};
use http::HeaderValue;
use http_body_util::Full;
use hyper::body::Bytes;
use hyper::{Request, Response};

const SHADOW_TEST_ROLE_REFERENCE: HeaderValue = HeaderValue::from_static("reference");
const SHADOW_TEST_ROLE_UPSTREAM: HeaderValue = HeaderValue::from_static("upstream");

pub struct Service {
    client: Client,
    dispatcher: Dispatcher,
    mirror: Mirror,
}

impl Service {
    pub fn new(dispatcher: Dispatcher, mirror: Mirror) -> Self {
        Self {
            dispatcher,
            client: hyper_util::client::legacy::Client::builder(
                hyper_util::rt::TokioExecutor::new(),
            )
            .build_http(),
            mirror,
        }
    }

    /// handle a request.
    /// this runs for any request (to get the reference), so it tries to do as little as possible
    pub async fn handle(
        &self,
        mut req: Request<Bytes>,
    ) -> Result<Response<Full<Bytes>>, error::Upstream> {
        let context = self.dispatcher.init_context(&req);

        // determine the Role-header
        let role = match &context.mode {
            RequestMode::Proxy => SHADOW_TEST_ROLE_UPSTREAM,
            RequestMode::Experiment { .. } => SHADOW_TEST_ROLE_REFERENCE,
        };

        // conditionally spawn a mirror-task
        self.mirror.spawn(context.mode);

        req.headers_mut().insert(SHADOW_TEST_HEADER, role);
        let response = self.client.upstream(req, &context.reference_uri).await;

        // send the reference-response over to the candidate-task
        context.tx.send_reference(context.reference_uri, &response);

        response.map(|r| r.map(Full::new))
    }
}
