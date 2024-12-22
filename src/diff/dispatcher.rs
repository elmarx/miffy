use crate::domain;
use crate::http::model::{RequestContext, RequestMode};
use bytes::Bytes;
use http::uri::PathAndQuery;
use tokio::sync::oneshot;

/// the dispatcher decides where to send the request, i.e. who is reference, who is candidate, test anything at all
pub struct Dispatcher {
    candidate_base: String,
    reference_base: String,
    router: matchit::Router<bool>,
}

impl Dispatcher {
    pub fn new(candidate_base: String, reference_base: String, routes: &[&str]) -> Self {
        let mut router = matchit::Router::new();

        for &r in routes {
            router.insert(r, true).expect("invalid path provided");
        }

        Self {
            candidate_base,
            reference_base,
            router,
        }
    }

    /// init a request-context. Decide if this is a request under test, or a normal request,
    /// and initialize all the required data
    pub fn init_context(&self, req: &http::Request<Bytes>) -> RequestContext {
        let uri = req.uri();

        let is_shadow_test = self.router.at(uri.path()).is_ok_and(|it| *it.value);
        let path_query = uri
            .path_and_query()
            .map_or(uri.path(), PathAndQuery::as_str);

        let reference_uri = format!("{}{}", self.reference_base, path_query);

        if !is_shadow_test {
            return RequestContext {
                reference_uri,
                tx: None,
                mode: RequestMode::Proxy,
            };
        }

        let candidate_uri = format!("{}{}", self.candidate_base, path_query);
        let (tx, rx) = oneshot::channel::<domain::Result>();

        RequestContext {
            reference_uri,
            tx: Some(tx),
            mode: RequestMode::Experiment {
                request: req.clone(),
                candidate_uri,
                rx,
            },
        }
    }
}
