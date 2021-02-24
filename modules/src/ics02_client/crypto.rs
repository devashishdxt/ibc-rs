use std::convert::TryFrom;

use ibc_proto::cosmos::crypto::{
    ed25519::PubKey as RawEd25519PubKey, secp256k1::PubKey as RawSecp256k1PubKey,
};
use prost_types::Any;
use serde::Serialize;
use tendermint_proto::Protobuf;

use crate::ics02_client::error::{Error, Kind};

pub const SECP256K1_PUB_KEY_TYPE_URL: &str = "/cosmos.crypto.secp256k1.PubKey";
pub const ED25519_PUB_KEY_TYPE_URL: &str = "/cosmos.crypto.ed25519.PubKey";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum AnyPublicKey {
    Secp256k1(Secp256k1PubKey),
    Ed25519(Ed25519PubKey),
}

impl AnyPublicKey {
    pub fn bytes(&self) -> &[u8] {
        match self {
            Self::Secp256k1(Secp256k1PubKey { ref key }) => key,
            Self::Ed25519(Ed25519PubKey { ref key }) => key,
        }
    }
}

impl Protobuf<Any> for AnyPublicKey {}

impl TryFrom<Any> for AnyPublicKey {
    type Error = Error;

    fn try_from(raw: Any) -> Result<Self, Self::Error> {
        match raw.type_url.as_str() {
            SECP256K1_PUB_KEY_TYPE_URL => Ok(AnyPublicKey::Secp256k1(
                Secp256k1PubKey::decode_vec(&raw.value)
                    .map_err(|e| Kind::InvalidRawPubKey.context(e))?,
            )),
            ED25519_PUB_KEY_TYPE_URL => Ok(AnyPublicKey::Ed25519(
                Ed25519PubKey::decode_vec(&raw.value)
                    .map_err(|e| Kind::InvalidRawPubKey.context(e))?,
            )),
            _ => Err(Kind::UnknownPubKeyType(raw.type_url).into()),
        }
    }
}

impl From<AnyPublicKey> for Any {
    fn from(value: AnyPublicKey) -> Self {
        match value {
            AnyPublicKey::Secp256k1(key) => Any {
                type_url: SECP256K1_PUB_KEY_TYPE_URL.to_string(),
                value: key.encode_vec().unwrap(),
            },
            AnyPublicKey::Ed25519(key) => Any {
                type_url: ED25519_PUB_KEY_TYPE_URL.to_string(),
                value: key.encode_vec().unwrap(),
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Secp256k1PubKey {
    key: Vec<u8>,
}

impl Protobuf<RawSecp256k1PubKey> for Secp256k1PubKey {}

impl From<RawSecp256k1PubKey> for Secp256k1PubKey {
    fn from(raw: RawSecp256k1PubKey) -> Self {
        Self { key: raw.key }
    }
}

impl From<Secp256k1PubKey> for RawSecp256k1PubKey {
    fn from(value: Secp256k1PubKey) -> Self {
        Self { key: value.key }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Ed25519PubKey {
    key: Vec<u8>,
}

impl Protobuf<RawEd25519PubKey> for Ed25519PubKey {}

impl From<RawEd25519PubKey> for Ed25519PubKey {
    fn from(raw: RawEd25519PubKey) -> Self {
        Self { key: raw.key }
    }
}

impl From<Ed25519PubKey> for RawEd25519PubKey {
    fn from(value: Ed25519PubKey) -> Self {
        Self { key: value.key }
    }
}
