use std::convert::{TryFrom, TryInto};

use ibc_proto::ibc::lightclients::solomachine::v1::Header as RawHeader;
use serde::Serialize;
use tendermint_proto::Protobuf;

use crate::{
    ics02_client::{
        client_def::AnyHeader, client_type::ClientType, crypto::AnyPublicKey,
        header::Header as IHeader,
    },
    ics07_tendermint::error::{Error, Kind},
    Height,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Header {
    pub sequence: u64,
    pub timestamp: u64,
    pub signature: Vec<u8>,
    pub new_public_key: AnyPublicKey,
    pub new_diversifier: String,
}

impl Header {
    pub fn new(
        sequence: u64,
        timestamp: u64,
        signature: Vec<u8>,
        new_public_key: AnyPublicKey,
        new_diversifier: String,
    ) -> Self {
        Self {
            sequence,
            timestamp,
            signature,
            new_public_key,
            new_diversifier,
        }
    }
}

impl IHeader for Header {
    fn client_type(&self) -> ClientType {
        ClientType::SoloMachine
    }

    fn height(&self) -> Height {
        Height {
            revision_number: 0,
            revision_height: self.sequence,
        }
    }

    fn validate_basic(&self) -> Result<(), Box<dyn std::error::Error>> {
        if self.sequence == 0 {
            return Err(Kind::InvalidHeader
                .context("sequence number cannot be zero")
                .into());
        }

        if self.timestamp == 0 {
            return Err(Kind::InvalidHeader
                .context("timestamp cannot be zero")
                .into());
        }

        if self.new_diversifier.trim().is_empty() {
            return Err(Kind::InvalidHeader
                .context("diversifier cannot contain only spaces")
                .into());
        }

        if self.new_public_key.bytes().is_empty() {
            return Err(Kind::InvalidHeader
                .context("new public key cannot be empty")
                .into());
        }

        Ok(())
    }

    fn wrap_any(self) -> AnyHeader {
        AnyHeader::SoloMachine(self)
    }
}

impl Protobuf<RawHeader> for Header {}

impl TryFrom<RawHeader> for Header {
    type Error = Error;

    fn try_from(value: RawHeader) -> Result<Self, Self::Error> {
        if value.new_public_key.is_none() {
            return Err(Kind::InvalidRawHeader.context("missing public key").into());
        }

        let new_public_key: AnyPublicKey = value
            .new_public_key
            .unwrap()
            .try_into()
            .map_err(|e| Kind::InvalidRawHeader.context(e))?;

        Ok(Self {
            sequence: value.sequence,
            timestamp: value.timestamp,
            signature: value.signature,
            new_public_key,
            new_diversifier: value.new_diversifier,
        })
    }
}

impl From<Header> for RawHeader {
    fn from(value: Header) -> Self {
        RawHeader {
            sequence: value.sequence,
            timestamp: value.timestamp,
            signature: value.signature,
            new_public_key: Some(value.new_public_key.into()),
            new_diversifier: value.new_diversifier,
        }
    }
}
