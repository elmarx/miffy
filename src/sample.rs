use hyper::body::Bytes;
use hyper::{Request, Response};

/// sample represents a shadow-tested request, i.e. a mirrored request that may be analyzed further
pub struct Sample {
    request: Request<Bytes>,
    reference: Response<Bytes>,
    candidate: Response<Bytes>,
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
        let (candidate_header, candidate_body) = self.candidate.into_parts();
        let (reference_header, reference_body) = self.reference.into_parts();

        let req = self.request;

        Some(format!("Request: {req:?}\n Candidate: {candidate_header:?} {candidate_body:#?}\nReference: {reference_header:?} {reference_body:#?}"))
    }
}
