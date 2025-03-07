use crate::settings::Route as ConfigRoute;
use crate::util::jaq::JaqFilter;
use std::sync::Arc;

pub struct Route {
    /// path in matchit-syntax (<https://docs.rs/matchit/latest/matchit/#parameters>). Names of parameters are irrelevant
    pub path: String,

    /// name of the param (from the route) to use as
    pub key: Option<String>,

    pub reference_base: Option<String>,
    pub candidate_base: Option<String>,

    pub reference_filter: Option<Arc<JaqFilter>>,
    pub candidate_filter: Option<Arc<JaqFilter>>,
}

impl TryFrom<&ConfigRoute> for Route {
    type Error = crate::util::jaq::Error;

    fn try_from(value: &ConfigRoute) -> Result<Self, Self::Error> {
        let candidate_filter = value
            .candidate_filter
            .as_ref()
            .map(|s| JaqFilter::try_from(s.as_str()))
            .transpose()?;
        let reference_filter = value
            .reference_filter
            .as_ref()
            .map(|s| JaqFilter::try_from(s.as_str()))
            .transpose()?;

        Ok(Self {
            path: value.path.clone(),
            key: value.key.clone(),
            reference_base: value.reference.clone(),
            candidate_base: value.candidate.clone(),
            reference_filter: reference_filter.map(Arc::new),
            candidate_filter: candidate_filter.map(Arc::new),
        })
    }
}
