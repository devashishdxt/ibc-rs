use std::convert::TryFrom;

use ibc_proto::ibc::lightclients::solomachine::v1::Misbehaviour as RawMisbehaviour;
use serde::Serialize;
use tendermint_proto::Protobuf;

use super::{
    error::{Error, Kind},
    signature_and_data::SignatureAndData,
};
use crate::{
    ics02_client::{
        client_type::ClientType, identifier_validator::validate_client_id,
        misbehaviour::Misbehaviour as IMisbehaviour,
    },
    Height,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Misbehaviour {
    pub client_id: String,
    pub sequence: u64,
    pub signature_one: SignatureAndData,
    pub signature_two: SignatureAndData,
}

impl IMisbehaviour for Misbehaviour {
    fn client_type(&self) -> ClientType {
        ClientType::SoloMachine
    }

    fn height(&self) -> Height {
        Height::new(0, self.sequence)
    }

    fn validate_basic(&self) -> Result<(), Box<dyn std::error::Error>> {
        validate_client_id(&self.client_id).map_err(|e| {
            Kind::InvalidMisbehaviour
                .context(format!("invalid client identifier for solo machine: {}", e))
        })?;

        if self.sequence == 0 {
            return Err(Kind::InvalidMisbehaviour
                .context("sequence cannot be 0")
                .into());
        }

        self.signature_one.validate_basic().map_err(|e| {
            Kind::InvalidMisbehaviour
                .context(format!("signature one failed basic validation: {}", e))
        })?;
        self.signature_two.validate_basic().map_err(|e| {
            Kind::InvalidMisbehaviour
                .context(format!("signature two failed basic validation: {}", e))
        })?;

        if self.signature_one == self.signature_two {
            return Err(Kind::InvalidMisbehaviour
                .context("misbehaviour signatures cannot be equal")
                .into());
        }

        if self.signature_one.data == self.signature_two.data {
            return Err(Kind::InvalidMisbehaviour
                .context("misbehaviour signature data must be signed over different messages")
                .into());
        }

        Ok(())
    }
}

impl Protobuf<RawMisbehaviour> for Misbehaviour {}

impl TryFrom<RawMisbehaviour> for Misbehaviour {
    type Error = Error;

    fn try_from(value: RawMisbehaviour) -> Result<Self, Self::Error> {
        let signature_one = value
            .signature_one
            .ok_or_else(|| Kind::InvalidRawMisbehaviour.context("missing signature one"))?
            .into();

        let signature_two = value
            .signature_two
            .ok_or_else(|| Kind::InvalidRawMisbehaviour.context("missing signature two"))?
            .into();

        Ok(Self {
            client_id: value.client_id,
            sequence: value.sequence,
            signature_one,
            signature_two,
        })
    }
}

impl From<Misbehaviour> for RawMisbehaviour {
    fn from(value: Misbehaviour) -> Self {
        Self {
            client_id: value.client_id,
            sequence: value.sequence,
            signature_one: Some(value.signature_one.into()),
            signature_two: Some(value.signature_two.into()),
        }
    }
}
