use std::convert::{TryFrom, TryInto};

use ibc_proto::cosmos::crypto::{
    ed25519::PubKey as RawEd25519PubKey, multisig::LegacyAminoPubKey as RawMultisigPubKey,
    secp256k1::PubKey as RawSecp256k1PubKey,
};
use prost_types::Any;
use serde::Serialize;
use tendermint_proto::Protobuf;

use crate::ics02_client::error::{Error, Kind};

pub const SECP256K1_PUB_KEY_TYPE_URL: &str = "/cosmos.crypto.secp256k1.PubKey";
pub const ED25519_PUB_KEY_TYPE_URL: &str = "/cosmos.crypto.ed25519.PubKey";
pub const MULTISIG_PUB_KEY_TYPE_URL: &str = "/cosmos.crypto.multisig.LegacyAminoPubKey";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum AnyPublicKey {
    Secp256k1(Secp256k1PubKey),
    Ed25519(Ed25519PubKey),
    Multisig(MultisigPubKey),
}

impl AnyPublicKey {
    pub fn is_empty(&self) -> bool {
        match self {
            Self::Secp256k1(Secp256k1PubKey { ref key }) => key.is_empty(),
            Self::Ed25519(Ed25519PubKey { ref key }) => key.is_empty(),
            Self::Multisig(key) => key.encode_vec().unwrap().is_empty(), // TODO: optimize
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
            AnyPublicKey::Multisig(key) => Any {
                type_url: MULTISIG_PUB_KEY_TYPE_URL.to_string(),
                value: key.encode_vec().unwrap(),
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Secp256k1PubKey {
    key: Vec<u8>,
}

impl Secp256k1PubKey {
    pub fn verify_signature(&self, msg: &[u8], signature: &[u8]) -> Result<(), Error> {
        let verify_key = k256::ecdsa::VerifyingKey::from_sec1_bytes(self.key.as_ref())
            .map_err(|e| Kind::InvalidPubKey.context(e))?;

        let signature = k256::ecdsa::Signature::try_from(signature)
            .map_err(|e| Kind::InvalidSignatureData.context(e))?;

        k256::ecdsa::signature::Verifier::verify(&verify_key, msg, &signature)
            .map_err(|e| Kind::SignatureVerificationFailed.context(e).into())
    }
}

impl AsRef<[u8]> for Secp256k1PubKey {
    fn as_ref(&self) -> &[u8] {
        &self.key
    }
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

impl Ed25519PubKey {
    pub fn verify_signature(&self, msg: &[u8], signature: &[u8]) -> Result<(), Error> {
        if signature.len() != 64 {
            return Err(Kind::InvalidSignatureData.into());
        }

        let mut sig = [0; 64];
        sig.copy_from_slice(&signature);

        let signature = ed25519_dalek::Signature::new(sig);
        let public_key = ed25519_dalek::PublicKey::from_bytes(self.key.as_ref())
            .map_err(|e| Kind::InvalidPubKey.context(e))?;

        ed25519_dalek::Verifier::verify(&public_key, msg, &signature)
            .map_err(|e| Kind::SignatureVerificationFailed.context(e).into())
    }
}

impl AsRef<[u8]> for Ed25519PubKey {
    fn as_ref(&self) -> &[u8] {
        &self.key
    }
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MultisigPubKey {
    pub threshold: u32,
    pub public_keys: Vec<AnyPublicKey>,
}

impl Protobuf<RawMultisigPubKey> for MultisigPubKey {}

impl TryFrom<RawMultisigPubKey> for MultisigPubKey {
    type Error = Error;

    fn try_from(value: RawMultisigPubKey) -> Result<Self, Self::Error> {
        Ok(Self {
            threshold: value.threshold,
            public_keys: value
                .public_keys
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<Vec<AnyPublicKey>, Error>>()?,
        })
    }
}

impl From<MultisigPubKey> for RawMultisigPubKey {
    fn from(value: MultisigPubKey) -> Self {
        Self {
            threshold: value.threshold,
            public_keys: value.public_keys.into_iter().map(Into::into).collect(),
        }
    }
}
