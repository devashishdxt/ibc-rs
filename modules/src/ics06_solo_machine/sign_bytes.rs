use ibc_proto::ibc::lightclients::solomachine::v1::{
    DataType as RawDataType, SignBytes as RawSignBytes,
};
use serde::Serialize;
use tendermint_proto::Protobuf;

use super::{header::Header, header_data::HeaderData};

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
        let data_type = value.data_type.into();

        let mut raw = Self {
            sequence: value.sequence,
            timestamp: value.timestamp,
            diversifier: value.diversifier,
            data_type: 0,
            data: value.data,
        };

        raw.set_data_type(data_type);

        raw
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum DataType {
    /// Default State
    UninitializedUnspecified,
    /// Data type for client state verification
    ClientState,
    /// Data type for consensus state verification
    ConsensusState,
    /// Data type for connection state verification
    ConnectionState,
    /// Data type for channel state verification
    ChannelState,
    /// Data type for packet commitment verification
    PacketCommitment,
    /// Data type for packet acknowledgement verification
    PacketAcknowledgement,
    /// Data type for packet receipt absence verification
    PacketReceiptAbsence,
    /// Data type for next sequence recv verification
    NextSequenceRecv,
    /// Data type for header verification
    Header,
}

impl From<DataType> for RawDataType {
    fn from(data_type: DataType) -> Self {
        match data_type {
            DataType::UninitializedUnspecified => Self::UninitializedUnspecified,
            DataType::ClientState => Self::ClientState,
            DataType::ConsensusState => Self::ConsensusState,
            DataType::ConnectionState => Self::ConnectionState,
            DataType::ChannelState => Self::ChannelState,
            DataType::PacketCommitment => Self::PacketCommitment,
            DataType::PacketAcknowledgement => Self::PacketAcknowledgement,
            DataType::PacketReceiptAbsence => Self::PacketReceiptAbsence,
            DataType::NextSequenceRecv => Self::NextSequenceRecv,
            DataType::Header => Self::Header,
        }
    }
}

impl From<RawDataType> for DataType {
    fn from(raw: RawDataType) -> Self {
        match raw {
            RawDataType::UninitializedUnspecified => Self::UninitializedUnspecified,
            RawDataType::ClientState => Self::ClientState,
            RawDataType::ConsensusState => Self::ConsensusState,
            RawDataType::ConnectionState => Self::ConnectionState,
            RawDataType::ChannelState => Self::ChannelState,
            RawDataType::PacketCommitment => Self::PacketCommitment,
            RawDataType::PacketAcknowledgement => Self::PacketAcknowledgement,
            RawDataType::PacketReceiptAbsence => Self::PacketReceiptAbsence,
            RawDataType::NextSequenceRecv => Self::NextSequenceRecv,
            RawDataType::Header => Self::Header,
        }
    }
}
