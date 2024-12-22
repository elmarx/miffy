use crate::proxy::error::Upstream;
use crate::representation::{RequestRepr, ResultRepr};
use hyper::body::Bytes;
use hyper::{Request, Response};
use serde::Serialize;

pub type Result = std::result::Result<Response<Bytes>, Error>;

/// sample represents a shadow-tested request, i.e. a mirrored request that may be analyzed further
pub struct Sample {
    request: Request<Bytes>,
    reference: Result,
    candidate: Result,
}

#[derive(Serialize)]
pub struct SampleMessage {
    request: RequestRepr,
    reference: ResultRepr,
    candidate: ResultRepr,
}

impl From<Sample> for SampleMessage {
    fn from(value: Sample) -> Self {
        let candidate = value.candidate.into();
        let reference = value.reference.into();
        let request = value.request.into();

        Self {
            candidate,
            reference,
            request,
        }
    }
}

impl Sample {
    pub fn new(request: Request<Bytes>, reference: Result, candidate: Result) -> Self {
        Self {
            request,
            reference,
            candidate,
        }
    }

    pub fn message(self) -> Option<String> {
        let message: SampleMessage = self.into();
        let msg = serde_json::to_string(&message).expect("failed to serialize sample-message");
        Some(msg)
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Error {
    Uri,
    Request,
    Body,
}

impl From<&Upstream> for Error {
    fn from(value: &Upstream) -> Self {
        match value {
            Upstream::InvalidUri(_) => Error::Uri,
            Upstream::Request(_) => Error::Request,
            Upstream::ReadBody(_) => Error::Body,
        }
    }
}
