use ibc_proto::cosmos::tx::signing::v1beta1::signature_descriptor::Data as Signature;
use prost::Message;

use super::{
    client_state::ClientState, consensus_state::ConsensusState, error::Kind, header::Header,
};
use crate::{
    ics02_client::{
        client_def::{AnyClientState, AnyConsensusState, ClientDef},
        crypto::verify_signature,
    },
    ics03_connection::connection::ConnectionEnd,
    ics04_channel::channel::ChannelEnd,
    ics23_commitment::commitment::{CommitmentPrefix, CommitmentProofBytes, CommitmentRoot},
    ics24_host::identifier::{ChannelId, ClientId, ConnectionId, PortId},
    Height,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SoloMachineClient;

impl ClientDef for SoloMachineClient {
    type Header = Header;

    type ClientState = ClientState;

    type ConsensusState = ConsensusState;

    fn check_header_and_update_state(
        &self,
        mut client_state: Self::ClientState,
        header: Self::Header,
    ) -> Result<(Self::ClientState, Self::ConsensusState), Box<dyn std::error::Error>> {
        if header.sequence != client_state.sequence {
            return Err(Kind::InvalidHeader
                .context(format!(
                    "header sequence does not match the client state sequence ({} != {})",
                    header.sequence, client_state.sequence,
                ))
                .into());
        }

        if header.timestamp < client_state.consensus_state.timestamp {
            return Err(Kind::InvalidHeader
                .context(format!(
                    "header timestamp is less than to the consensus state timestamp ({} < {})",
                    header.timestamp, client_state.consensus_state.timestamp
                ))
                .into());
        }

        let header_sign_bytes = header.get_sign_bytes();
        let signature_data = Signature::decode(header.signature.as_ref())
            .map_err(|e| Kind::InvalidSignatureData.context(e))?
            .sum
            .ok_or_else(|| Kind::InvalidHeader.context("missing signature data"))?;
        let public_key = client_state.consensus_state.public_key;

        verify_signature(&public_key, &header_sign_bytes, &signature_data)?;

        client_state.consensus_state.public_key = header.new_public_key;
        client_state.consensus_state.diversifier = header.new_diversifier;
        client_state.consensus_state.timestamp = header.timestamp;
        client_state.sequence += 1;

        let consensus_state = client_state.consensus_state.clone();

        Ok((client_state, consensus_state))
    }

    fn verify_client_consensus_state(
        &self,
        _client_state: &Self::ClientState,
        _height: Height,
        _prefix: &CommitmentPrefix,
        _proof: &CommitmentProofBytes,
        _client_id: &ClientId,
        _consensus_height: Height,
        _expected_consensus_state: &AnyConsensusState,
    ) -> Result<(), Box<dyn std::error::Error>> {
        todo!("@devashishdxt")
    }

    fn verify_connection_state(
        &self,
        _client_state: &Self::ClientState,
        _height: Height,
        _prefix: &CommitmentPrefix,
        _proof: &CommitmentProofBytes,
        _connection_id: &ConnectionId,
        _expected_connection_end: &ConnectionEnd,
    ) -> Result<(), Box<dyn std::error::Error>> {
        todo!("@devashishdxt")
    }

    fn verify_channel_state(
        &self,
        _client_state: &Self::ClientState,
        _height: Height,
        _prefix: &CommitmentPrefix,
        _proof: &CommitmentProofBytes,
        _port_id: &PortId,
        _channel_id: &ChannelId,
        _expected_channel_end: &ChannelEnd,
    ) -> Result<(), Box<dyn std::error::Error>> {
        todo!("@devashishdxt")
    }

    fn verify_client_full_state(
        &self,
        _client_state: &Self::ClientState,
        _height: Height,
        _root: Option<&CommitmentRoot>,
        _prefix: &CommitmentPrefix,
        _client_id: &ClientId,
        _proof: &CommitmentProofBytes,
        _counterparty_client_state: &AnyClientState,
    ) -> Result<(), Box<dyn std::error::Error>> {
        todo!("@devashishdxt")
    }
}
