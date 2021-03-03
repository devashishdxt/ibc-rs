use std::convert::TryFrom;

use ibc_proto::cosmos::tx::signing::v1beta1::signature_descriptor::{
    data::{Multi, Single, Sum as SignatureData},
    Data,
};
use k256::ecdsa::{signature::Verifier, Signature, VerifyingKey};
use prost::Message;

use super::{
    client_state::ClientState,
    consensus_state::ConsensusState,
    error::{Error, Kind},
    header::Header,
};
use crate::{
    ics02_client::{
        client_def::{AnyClientState, AnyConsensusState, ClientDef},
        crypto::{AnyPublicKey, Ed25519PubKey, MultisigPubKey, Secp256k1PubKey},
    },
    ics04_channel::channel::ChannelEnd,
    ics23_commitment::commitment::{CommitmentPrefix, CommitmentProofBytes, CommitmentRoot},
    ics24_host::identifier::{ChannelId, ClientId, PortId},
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
        let signature_data = Data::decode(header.signature.as_ref())
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
        _connection_id: &crate::ics24_host::identifier::ConnectionId,
        _expected_connection_end: &crate::ics03_connection::connection::ConnectionEnd,
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

fn verify_signature(
    public_key: &AnyPublicKey,
    msg: &[u8],
    signature: &SignatureData,
) -> Result<(), Error> {
    match (public_key, signature) {
        (AnyPublicKey::Secp256k1(ref public_key), SignatureData::Single(ref signature_data)) => {
            verify_secp256k1_signature(public_key, msg, signature_data)
        }
        (AnyPublicKey::Ed25519(ref public_key), SignatureData::Single(ref signature_data)) => {
            verify_ed25519_signature(public_key, msg, signature_data)
        }
        (AnyPublicKey::Multisig(ref public_key), SignatureData::Multi(ref signature_data)) => {
            verify_multi_signature(public_key, msg, signature_data)
        }
        _ => Err(Kind::InvalidHeader.context("invalid signature type").into()),
    }
}

fn verify_secp256k1_signature(
    public_key: &Secp256k1PubKey,
    msg: &[u8],
    signature_data: &Single,
) -> Result<(), Error> {
    let verify_key = VerifyingKey::from_sec1_bytes(public_key.as_ref())
        .map_err(|e| Kind::InvalidPubKey.context(e))?;

    let signature = Signature::try_from(signature_data.signature.as_ref())
        .map_err(|e| Kind::InvalidSignatureData.context(e))?;

    verify_key
        .verify(msg, &signature)
        .map_err(|e| Kind::SignatureVerificationFailed.context(e))?;

    Ok(())
}

fn verify_ed25519_signature(
    _public_key: &Ed25519PubKey,
    _msg: &[u8],
    _signature_data: &Single,
) -> Result<(), Error> {
    todo!("@devashishdxt")
}

fn verify_multi_signature(
    _public_key: &MultisigPubKey,
    _msg: &[u8],
    _signature_data: &Multi,
) -> Result<(), Error> {
    todo!("@devashishdxt")
}
