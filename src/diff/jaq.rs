use crate::domain::{Body, RequestResult};
use crate::jaq;
use crate::jaq::JaqFilter;
use crate::model::Filter;

pub fn transform_a(filter: Option<&JaqFilter>, result: RequestResult) -> RequestResult {
    todo!()
}

pub fn transform(filter: &JaqFilter, body: &Body) -> Option<Result<Body, jaq::Error>> {
    match body {
        Body::Json(b) => Some(filter.run(b.clone()).map(Body::Json)),
        _ => None,
    }
}

impl Filter {
    pub fn pre(&self, result: RequestResult) -> Result<RequestResult, jaq::Error> {
        if let Some(filter) = &self.pre {
            let transformed = transform(filter, )
        } else {
            Ok(result)
        }
    }
}
