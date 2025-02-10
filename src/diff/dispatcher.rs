use crate::domain;
use crate::http::model::{RequestContext, RequestMode};
use crate::settings::Route;
use bytes::Bytes;
use http::uri::PathAndQuery;
use tokio::sync::oneshot;

/// the dispatcher decides where to send the request, i.e. who is reference, who is candidate, test anything at all
pub struct Dispatcher {
    default_candidate_base: String,
    default_reference_base: String,
    router: matchit::Router<Route>,
}

impl Dispatcher {
    pub fn new(
        default_reference_base: String,
        default_candidate_base: String,
        routes: &[Route],
    ) -> Self {
        let mut router = matchit::Router::new();

        for r in routes {
            router
                .insert(&r.path, r.clone())
                .expect("invalid path provided");
        }

        Self {
            default_candidate_base,
            default_reference_base,
            router,
        }
    }

    fn init_context_for_test(
        &self,
        request: &http::Request<Bytes>,
        path_query: &str,
        r: &Route,
    ) -> RequestContext {
        let (tx, rx) = oneshot::channel::<domain::RequestResult>();

        let reference_base = r.reference.as_ref().unwrap_or(&self.default_reference_base);
        let candidate_base = r.candidate.as_ref().unwrap_or(&self.default_candidate_base);

        let reference_uri = format!("{reference_base}{path_query}");
        let candidate_uri = format!("{candidate_base}{path_query}");

        RequestContext {
            reference_uri,
            tx: Some(tx),
            mode: RequestMode::Experiment {
                path: r.path.clone(),
                request: request.clone(),
                candidate_uri,
                rx,
            },
        }
    }

    /// init a request-context. Decide if this is a request under test, or a normal request,
    /// and initialize all the required data
    pub fn init_context(&self, req: &http::Request<Bytes>) -> RequestContext {
        let uri = req.uri();

        let parameters = self.router.at(uri.path()).ok();

        let path_query = uri
            .path_and_query()
            .map_or(uri.path(), PathAndQuery::as_str);

        if let Some(m) = parameters {
            self.init_context_for_test(req, path_query, m.value)
        } else {
            let reference_uri = format!("{}{}", self.default_reference_base, path_query);
            RequestContext {
                reference_uri,
                tx: None,
                mode: RequestMode::Proxy,
            }
        }
    }
}
