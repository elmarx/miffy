use crate::representation::{RequestRepr, ResponseRepr};
use hyper::body::Bytes;
use hyper::{Request, Response};
use serde::Serialize;

/// sample represents a shadow-tested request, i.e. a mirrored request that may be analyzed further
pub struct Sample {
    request: Request<Bytes>,
    reference: Response<Bytes>,
    candidate: Response<Bytes>,
}

#[derive(Serialize)]
pub struct SampleMessage {
    request: RequestRepr,
    reference: ResponseRepr,
    candidate: ResponseRepr,
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
    pub fn new(
        request: Request<Bytes>,
        reference: Response<Bytes>,
        candidate: Response<Bytes>,
    ) -> Self {
        Self {
            request,
            reference,
            candidate,
        }
    }

    pub fn message(self) -> Option<String> {
        let message: SampleMessage = self.into();
        let msg = serde_json::to_string(&message).unwrap();
        Some(msg)
    }
}
