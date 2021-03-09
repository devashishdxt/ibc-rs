use ibc_proto::ibc::lightclients::solomachine::v1::SignBytes as RawSignBytes;
use serde::Serialize;
use tendermint_proto::Protobuf;

use super::{data_type::DataType, header::Header, header_data::HeaderData};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SignBytes {
    pub sequence: u64,
    pub timestamp: u64,
    pub diversifier: String,
    pub data_type: DataType,
    pub data: Vec<u8>,
}

impl From<&Header> for SignBytes {
    fn from(header: &Header) -> Self {
        let header_data: HeaderData = header.into();
        let header_data_bytes = header_data.encode_vec().unwrap();

        SignBytes {
            sequence: header.sequence,
            timestamp: header.timestamp,
            diversifier: header.new_diversifier.clone(),
            data_type: DataType::Header,
            data: header_data_bytes,
        }
    }
}

impl Protobuf<RawSignBytes> for SignBytes {}

impl From<RawSignBytes> for SignBytes {
    fn from(raw: RawSignBytes) -> Self {
        let data_type = raw.data_type().into();

        Self {
            sequence: raw.sequence,
            timestamp: raw.timestamp,
            diversifier: raw.diversifier,
            data_type,
            data: raw.data,
        }
    }
}

impl From<SignBytes> for RawSignBytes {
    fn from(value: SignBytes) -> Self {
        let mut raw = Self {
            sequence: value.sequence,
            timestamp: value.timestamp,
            diversifier: value.diversifier,
            data_type: 0,
            data: value.data,
        };

        raw.set_data_type(value.data_type.into());

        raw
    }
}
