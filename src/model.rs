use crate::jaq;
use crate::jaq::JaqFilter;
use crate::settings::Route as ConfigRoute;
use crate::settings::Upstream as ConfigUpstream;
use std::sync::Arc;

pub struct Route {
    /// path in matchit-syntax (<https://docs.rs/matchit/latest/matchit/#parameters>). Names of parameters are irrelevant
    pub path: String,

    /// name of the param (from the route) to use as
    pub key: Option<String>,

    pub reference_base: Option<String>,
    pub candidate_base: Option<String>,

    pub reference_filter: Option<Arc<Filter>>,
    pub candidate_filter: Option<Arc<Filter>>,
}

pub struct Filter {
    /// transform immediately
    pub pre: Option<JaqFilter>,
    /// transform after comparison
    pub post: Option<JaqFilter>,
    /// customize comparison
    pub canonicalize: Option<JaqFilter>,
}

fn compile(filter: &Option<String>) -> Result<Option<JaqFilter>, jaq::Error> {
    filter
        .as_ref()
        .map(|s| JaqFilter::try_from(s.as_str()))
        .transpose()
}

impl TryFrom<&ConfigUpstream> for Filter {
    type Error = jaq::Error;

    fn try_from(value: &ConfigUpstream) -> Result<Self, Self::Error> {
        let pre_transform = compile(&value.pre_transform)?;
        let post_transform = compile(&value.post_transform)?;
        let canonicalize = compile(&value.canonicalize)?;

        Ok(Self {
            pre: pre_transform,
            post: post_transform,
            canonicalize,
        })
    }
}

impl TryFrom<&ConfigRoute> for Route {
    type Error = jaq::Error;

    fn try_from(value: &ConfigRoute) -> Result<Self, Self::Error> {
        let reference = value
            .reference
            .as_ref()
            .map(Filter::try_from)
            .transpose()?
            .map(Arc::new);
        let candidate = value
            .candidate
            .as_ref()
            .map(Filter::try_from)
            .transpose()?
            .map(Arc::new);

        Ok(Self {
            path: value.path.clone(),
            key: value.key.clone(),
            reference_base: value.reference.as_ref().and_then(|r| r.base.clone()),
            candidate_base: value.candidate.as_ref().and_then(|r| r.base.clone()),
            reference_filter: reference,
            candidate_filter: candidate,
        })
    }
}
