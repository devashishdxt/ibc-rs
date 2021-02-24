use std::convert::{TryFrom, TryInto};

use ibc_proto::ibc::lightclients::solomachine::v1::ClientState as RawClientState;
use serde::Serialize;
use tendermint_proto::Protobuf;

use super::{
    consensus_state::ConsensusState,
    error::{Error, Kind},
};
use crate::{
    ics02_client::{
        client_def::AnyClientState,
        client_type::ClientType,
        state::{ClientState as IClientState, ConsensusState as _},
    },
    Height,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ClientState {
    pub sequence: u64,
    pub frozen_sequence: u64,
    pub consensus_state: ConsensusState,
    pub allow_update_after_proposal: bool,
}

impl ClientState {
    pub fn new(
        sequence: u64,
        frozen_sequence: u64,
        consensus_state: ConsensusState,
        allow_update_after_proposal: bool,
    ) -> Self {
        Self {
            sequence,
            frozen_sequence,
            consensus_state,
            allow_update_after_proposal,
        }
    }
}

impl IClientState for ClientState {
    fn client_type(&self) -> ClientType {
        ClientType::SoloMachine
    }

    fn latest_height(&self) -> Height {
        Height {
            revision_number: 0,
            revision_height: self.sequence,
        }
    }

    fn is_frozen(&self) -> bool {
        self.frozen_sequence != 0
    }

    fn validate_basic(&self) -> Result<(), Box<dyn std::error::Error>> {
        if self.sequence == 0 {
            return Err(Kind::InvalidClientState
                .context("sequence cannot be 0")
                .into());
        }

        self.consensus_state.validate_basic()
    }

    fn wrap_any(self) -> AnyClientState {
        AnyClientState::SoloMachine(self)
    }
}

impl Protobuf<RawClientState> for ClientState {}

impl TryFrom<RawClientState> for ClientState {
    type Error = Error;

    fn try_from(value: RawClientState) -> Result<Self, Self::Error> {
        if value.consensus_state.is_none() {
            return Err(Kind::InvalidRawClientState
                .context("missing consensus state")
                .into());
        }

        let consensus_state: ConsensusState = value
            .consensus_state
            .unwrap()
            .try_into()
            .map_err(|e| Kind::InvalidRawClientState.context(e))?;

        Ok(Self {
            sequence: value.sequence,
            frozen_sequence: value.frozen_sequence,
            consensus_state,
            allow_update_after_proposal: value.allow_update_after_proposal,
        })
    }
}

impl From<ClientState> for RawClientState {
    fn from(value: ClientState) -> Self {
        Self {
            sequence: value.sequence,
            frozen_sequence: value.frozen_sequence,
            consensus_state: Some(value.consensus_state.into()),
            allow_update_after_proposal: value.allow_update_after_proposal,
        }
    }
}
