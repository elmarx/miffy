use crate::http::model::{RequestContext, RequestMode};
use crate::model::Route;
use crate::settings::Route as ConfigRoute;
use anyhow::Context;
use bytes::Bytes;
use http::uri::PathAndQuery;
use matchit::Match;
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
        routes: &[ConfigRoute],
    ) -> anyhow::Result<Self> {
        let mut router = matchit::Router::new();

        for r in routes {
            let route = r.try_into().context("invalid route path")?;
            router
                .insert(&r.path, route)
                .expect("invalid path provided");
        }

        Ok(Self {
            default_candidate_base,
            default_reference_base,
            router,
        })
    }

    /// build the request-context with all data required to mirror traffic (and publish
    fn init_context_for_experiment(
        &self,
        request: &http::Request<Bytes>,
        path_query: &str,
        matched_route: &Match<&Route>,
    ) -> RequestContext {
        // remember: this runs on the main "thread", so do as little work as possible!
        let (tx, rx) = oneshot::channel();

        let route_value = matched_route.value;
        let params = matched_route
            .params
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();

        let reference_base = route_value
            .reference_base
            .as_ref()
            .unwrap_or(&self.default_reference_base);
        let candidate_base = route_value
            .candidate_base
            .as_ref()
            .unwrap_or(&self.default_candidate_base);

        let reference_uri = format!("{reference_base}{path_query}");
        let candidate_uri = format!("{candidate_base}{path_query}");

        RequestContext {
            reference_uri,
            tx: Some(tx),
            mode: RequestMode::Experiment {
                key: route_value.key.clone(),
                route: route_value.path.clone(),
                route_params: params,
                request: request.clone(),
                candidate_uri,
                rx,
                reference_filter: route_value.reference_filter.clone(),
                candidate_filter: route_value.candidate_filter.clone(),
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
            self.init_context_for_experiment(req, path_query, &m)
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
