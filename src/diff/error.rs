use crate::http::error::Upstream;
use thiserror::Error;
use tokio::sync::oneshot::error::RecvError;

#[derive(Debug, Error)]
pub enum Internal {
    #[error(transparent)]
    RecvError(#[from] RecvError),

    #[error(transparent)]
    Upstream(#[from] Upstream),
}
