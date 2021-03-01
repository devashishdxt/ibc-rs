use std::convert::{TryFrom, TryInto};

use ibc_proto::ibc::lightclients::solomachine::v1::HeaderData as RawHeaderData;
use serde::Serialize;
use tendermint_proto::Protobuf;

use super::{
    error::{Error, Kind},
    header::Header,
};
use crate::ics02_client::crypto::AnyPublicKey;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct HeaderData {
    pub new_public_key: AnyPublicKey,
    pub new_diversifier: String,
}

impl Protobuf<RawHeaderData> for HeaderData {}

impl TryFrom<RawHeaderData> for HeaderData {
    type Error = Error;

    fn try_from(value: RawHeaderData) -> Result<Self, Self::Error> {
        if value.new_pub_key.is_none() {
            return Err(Kind::InvalidRawHeaderData
                .context("missing public key")
                .into());
        }

        let new_public_key: AnyPublicKey = value
            .new_pub_key
            .unwrap()
            .try_into()
            .map_err(|e| Kind::InvalidRawHeaderData.context(e))?;

        Ok(Self {
            new_public_key,
            new_diversifier: value.new_diversifier,
        })
    }
}

impl From<HeaderData> for RawHeaderData {
    fn from(value: HeaderData) -> Self {
        RawHeaderData {
            new_pub_key: Some(value.new_public_key.into()),
            new_diversifier: value.new_diversifier,
        }
    }
}

impl From<&Header> for HeaderData {
    fn from(header: &Header) -> Self {
        Self {
            new_public_key: header.new_public_key.clone(),
            new_diversifier: header.new_diversifier.clone(),
        }
    }
}
