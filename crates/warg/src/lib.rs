use anyhow::Error;
use signature::Error as SignatureError;

mod proto_envelope;
mod serde_envelope;
pub mod operator;
pub mod package;
pub mod registry;
pub mod signing;

pub use proto_envelope::{ProtoEnvelope, ProtoEnvelopeBody};
pub use serde_envelope::SerdeEnvelope;

/// Types for converting to and from protobuf
pub mod protobuf {
    #![allow(clippy::all)]
    // Generated by [`prost-build`]
    include!(concat!(env!("OUT_DIR"), "/warg.rs"));
    // Generated by [`pbjson-build`]
    include!(concat!(env!("OUT_DIR"), "/warg.serde.rs"));

    pub fn prost_to_pbjson_timestamp(timestamp: prost_types::Timestamp) -> pbjson_types::Timestamp {
        pbjson_types::Timestamp {
            seconds: timestamp.seconds,
            nanos: timestamp.nanos,
        }
    }

    pub fn pbjson_to_prost_timestamp(timestamp: pbjson_types::Timestamp) -> prost_types::Timestamp {
        prost_types::Timestamp {
            seconds: timestamp.seconds,
            nanos: timestamp.nanos,
        }
    }
}

pub trait Signable: Encode {
    const PREFIX: &'static [u8];

    fn sign(
        &self,
        private_key: &signing::PrivateKey,
    ) -> Result<signing::Signature, SignatureError> {
        let prefixed_content = [Self::PREFIX, b":", self.encode().as_slice()].concat();
        private_key.sign(&prefixed_content)
    }

    fn verify(
        public_key: &signing::PublicKey,
        msg: &[u8],
        signature: &signing::Signature,
    ) -> Result<(), SignatureError> {
        let prefixed_content = [Self::PREFIX, b":", msg].concat();
        public_key.verify(&prefixed_content, signature)
    }
}

pub trait Decode: Sized {
    fn decode(bytes: &[u8]) -> Result<Self, Error>;
}

pub trait Encode {
    fn encode(&self) -> Vec<u8>;
}

/// Helper module for serializing and deserializing timestamps.
///
/// This is used over serde's built-in implementation to produce cleaner timestamps
/// in serialized output.
mod timestamp {
    use serde::Deserializer;
    use serde::{Deserialize, Serializer};
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    pub fn serialize<S>(timestamp: &SystemTime, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::Error;

        let duration_since_epoch = match timestamp.duration_since(UNIX_EPOCH) {
            Ok(duration_since_epoch) => duration_since_epoch,
            Err(_) => return Err(S::Error::custom("timestamp must be later than UNIX_EPOCH")),
        };

        serializer.serialize_str(&format!(
            "{secs}.{nsecs}",
            secs = duration_since_epoch.as_secs(),
            nsecs = duration_since_epoch.subsec_nanos()
        ))
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<SystemTime, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::Error;

        let s = String::deserialize(deserializer)?;
        let (secs, nsecs) = s
            .split_once('.')
            .ok_or_else(|| D::Error::custom("timestamp must be in the format <secs>.<nsecs>"))?;

        Ok(SystemTime::UNIX_EPOCH
            + Duration::new(
                secs.parse::<u64>().map_err(D::Error::custom)?,
                nsecs.parse::<u32>().map_err(D::Error::custom)?,
            ))
    }
}
