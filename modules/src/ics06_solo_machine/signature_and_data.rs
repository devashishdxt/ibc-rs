use ibc_proto::ibc::lightclients::solomachine::v1::SignatureAndData as RawSignatureAndData;
use serde::Serialize;
use tendermint_proto::Protobuf;

use super::{
    data_type::DataType,
    error::{Error, Kind},
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SignatureAndData {
    pub signature: Vec<u8>,
    pub data_type: DataType,
    pub data: Vec<u8>,
    pub timestamp: u64,
}

impl SignatureAndData {
    pub fn validate_basic(&self) -> Result<(), Error> {
        if self.signature.is_empty() {
            return Err(Kind::InvalidSignatureAndData
                .context("signature cannot be empty")
                .into());
        }

        if self.data.is_empty() {
            return Err(Kind::InvalidSignatureAndData
                .context("data for signature cannot be empty")
                .into());
        }

        if self.data_type == DataType::UninitializedUnspecified {
            return Err(Kind::InvalidSignatureAndData
                .context("data type cannot be UNSPECIFIED")
                .into());
        }

        if self.timestamp == 0 {
            return Err(Kind::InvalidSignatureAndData
                .context("timestamp cannot be 0")
                .into());
        }

        Ok(())
    }
}

impl Protobuf<RawSignatureAndData> for SignatureAndData {}

impl From<RawSignatureAndData> for SignatureAndData {
    fn from(value: RawSignatureAndData) -> Self {
        let data_type = value.data_type().into();

        Self {
            signature: value.signature,
            data_type,
            data: value.data,
            timestamp: value.timestamp,
        }
    }
}

impl From<SignatureAndData> for RawSignatureAndData {
    fn from(value: SignatureAndData) -> Self {
        let mut raw = Self {
            signature: value.signature,
            data_type: 0,
            data: value.data,
            timestamp: value.timestamp,
        };

        raw.set_data_type(value.data_type.into());

        raw
    }
}
