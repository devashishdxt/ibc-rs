use anomaly::{BoxError, Context};
use thiserror::Error;

pub type Error = anomaly::Error<Kind>;

#[derive(Clone, Debug, Error)]
pub enum Kind {
    #[error("invalid client state, failed basic validation")]
    InvalidClientState,

    #[error("invalid consensus state, failed basic validation")]
    InvalidConsensusState,

    #[error("invalid header, failed basic validation")]
    InvalidHeader,

    #[error("invalid raw client state")]
    InvalidRawClientState,

    #[error("invalid raw client consensus state")]
    InvalidRawConsensusState,

    #[error("invalid raw header")]
    InvalidRawHeader,

    #[error("invalid raw header data")]
    InvalidRawHeaderData,

    #[error("invalid signature data")]
    InvalidSignatureData,

    #[error("signature verification failed")]
    SignatureVerificationFailed,
}

impl Kind {
    pub fn context(self, source: impl Into<BoxError>) -> Context<Self> {
        Context::new(self, Some(source.into()))
    }
}
