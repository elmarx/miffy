use crate::diff::dispatcher::Dispatcher;
use crate::diff::mirror::Mirror;
use crate::diff::tx_ext::TxExt;
use crate::http::client::{Client, UpstreamExt};
use crate::http::error;
use http_body_util::Full;
use hyper::body::Bytes;
use hyper::{Request, Response};

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
        req: Request<Bytes>,
    ) -> Result<Response<Full<Bytes>>, error::Upstream> {
        let context = self.dispatcher.init_context(&req);
        // conditionally spawn a mirror-task
        self.mirror.spawn(context.mode);

        let response = self.client.upstream(req, &context.reference_uri).await;

        // send the reference-response over to the candidate-task
        context.tx.send_reference(context.reference_uri, &response);

        response.map(|r| r.map(Full::new))
    }
}
