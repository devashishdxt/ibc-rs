use std::convert::{TryFrom, TryInto};

use ibc_proto::ibc::lightclients::solomachine::v1::ConsensusState as RawConsensusState;
use serde::Serialize;
use tendermint_proto::Protobuf;

use crate::{
    ics02_client::{
        client_def::AnyConsensusState, client_type::ClientType, crypto::AnyPublicKey,
        state::ConsensusState as IConsensusState,
    },
    ics06_solo_machine::error::{Error, Kind},
    ics23_commitment::commitment::CommitmentRoot,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ConsensusState {
    pub public_key: AnyPublicKey,
    pub diversifier: String,
    pub timestamp: u64,
}

impl ConsensusState {
    pub fn new(public_key: AnyPublicKey, diversifier: String, timestamp: u64) -> Self {
        Self {
            public_key,
            diversifier,
            timestamp,
        }
    }
}

impl IConsensusState for ConsensusState {
    fn client_type(&self) -> ClientType {
        ClientType::SoloMachine
    }

    fn root(&self) -> Option<&CommitmentRoot> {
        None
    }

    fn validate_basic(&self) -> Result<(), Box<dyn std::error::Error>> {
        if self.timestamp == 0 {
            return Err(Kind::InvalidConsensusState
                .context("timestamp cannot be 0")
                .into());
        }

        if self.diversifier.trim().is_empty() {
            return Err(Kind::InvalidConsensusState
                .context("diversifier cannot contain only spaces")
                .into());
        }

        if self.public_key.bytes().is_empty() {
            return Err(Kind::InvalidConsensusState
                .context("public key cannot be empty")
                .into());
        }

        Ok(())
    }

    fn wrap_any(self) -> AnyConsensusState {
        AnyConsensusState::SoloMachine(self)
    }
}

impl Protobuf<RawConsensusState> for ConsensusState {}

impl TryFrom<RawConsensusState> for ConsensusState {
    type Error = Error;

    fn try_from(value: RawConsensusState) -> Result<Self, Self::Error> {
        if value.public_key.is_none() {
            return Err(Kind::InvalidRawConsensusState
                .context("missing public key")
                .into());
        }

        let public_key: AnyPublicKey = value
            .public_key
            .unwrap()
            .try_into()
            .map_err(|e| Kind::InvalidRawConsensusState.context(e))?;

        Ok(Self {
            public_key,
            diversifier: value.diversifier,
            timestamp: value.timestamp,
        })
    }
}

impl From<ConsensusState> for RawConsensusState {
    fn from(value: ConsensusState) -> Self {
        RawConsensusState {
            public_key: Some(value.public_key.into()),
            diversifier: value.diversifier,
            timestamp: value.timestamp,
        }
    }
}
