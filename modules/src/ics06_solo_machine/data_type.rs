use ibc_proto::ibc::lightclients::solomachine::v1::DataType as RawDataType;
use serde::Serialize;

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
