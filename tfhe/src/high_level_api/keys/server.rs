use tfhe_versionable::Versionize;

use crate::backward_compatibility::keys::{CompressedServerKeyVersions, ServerKeyVersions};
#[cfg(feature = "gpu")]
use crate::core_crypto::gpu::{synchronize_devices, CudaStreams};
use crate::high_level_api::keys::{IntegerCompressedServerKey, IntegerServerKey};
use crate::shortint::list_compression::{
    CompressedCompressionKey, CompressedDecompressionKey, CompressionKey, DecompressionKey,
};
use std::sync::Arc;

use super::ClientKey;

/// Key of the server
///
/// This key contains the different keys needed to be able to do computations for
/// each data type.
///
/// For a server to be able to do some FHE computations, the client needs to send this key
/// beforehand.
// Keys are stored in an Arc, so that cloning them is cheap
// (compared to an actual clone hundreds of MB / GB), and cheap cloning is needed for
// multithreading with less overhead)
#[derive(Clone, Versionize)]
#[versionize(ServerKeyVersions)]
pub struct ServerKey {
    pub(crate) key: Arc<IntegerServerKey>,
}

impl ServerKey {
    pub fn new(keys: &ClientKey) -> Self {
        Self {
            key: Arc::new(IntegerServerKey::new(&keys.key)),
        }
    }

    pub fn into_raw_parts(
        self,
    ) -> (
        crate::integer::ServerKey,
        Option<crate::integer::wopbs::WopbsKey>,
        Option<crate::integer::key_switching_key::KeySwitchingKeyMaterial>,
        Option<CompressionKey>,
        Option<DecompressionKey>,
    ) {
        let IntegerServerKey {
            key,
            wopbs_key,
            cpk_key_switching_key_material,
            compression_key,
            decompression_key,
        } = (*self.key).clone();

        (
            key,
            wopbs_key,
            cpk_key_switching_key_material,
            compression_key,
            decompression_key,
        )
    }

    pub fn from_raw_parts(
        key: crate::integer::ServerKey,
        wopbs_key: Option<crate::integer::wopbs::WopbsKey>,
        cpk_key_switching_key_material: Option<
            crate::integer::key_switching_key::KeySwitchingKeyMaterial,
        >,
        compression_key: Option<CompressionKey>,
        decompression_key: Option<DecompressionKey>,
    ) -> Self {
        Self {
            key: Arc::new(IntegerServerKey {
                key,
                wopbs_key,
                cpk_key_switching_key_material,
                compression_key,
                decompression_key,
            }),
        }
    }
}

impl AsRef<crate::integer::ServerKey> for ServerKey {
    fn as_ref(&self) -> &crate::integer::ServerKey {
        &self.key.key
    }
}

// By default, serde does not derives Serialize/Deserialize for `Rc` and `Arc` types
// as they can result in multiple copies, since serializing has to serialize the actual data
// not the pointer.
//
// serde has a `rc` feature to allow deriving on Arc and Rc types
// but activating it in our lib would mean also activate it for all the dependency stack,
// so tfhe-rs users would have this feature enabled on our behalf and we don't want that
// so we implement the serialization / deserialization ourselves.
//
// In the case of our ServerKey, this is fine, we expect programs to only
// serialize and deserialize the same server key only once.
// The inner `Arc` are used to make copying a server key more performant before a `set_server_key`
// in multi-threading scenarios.
#[derive(serde::Serialize)]
// We directly versionize the `ServerKey` without having to use this intermediate type.
#[cfg_attr(tfhe_lints, allow(tfhe_lints::serialize_without_versionize))]
struct SerializableServerKey<'a> {
    pub(crate) integer_key: &'a IntegerServerKey,
}

impl serde::Serialize for ServerKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        SerializableServerKey {
            integer_key: &self.key,
        }
        .serialize(serializer)
    }
}

#[derive(serde::Deserialize)]
struct DeserializableServerKey {
    pub(crate) integer_key: IntegerServerKey,
}

impl<'de> serde::Deserialize<'de> for ServerKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        DeserializableServerKey::deserialize(deserializer).map(|deserialized| Self {
            key: Arc::new(deserialized.integer_key),
        })
    }
}

/// Compressed ServerKey
///
/// A CompressedServerKey takes much less disk space / memory space than a
/// ServerKey.
///
/// It has to be decompressed into a ServerKey in order to be usable.
///
/// Once decompressed, it is not possible to recompress the key.
#[derive(Clone, serde::Serialize, serde::Deserialize, Versionize)]
#[versionize(CompressedServerKeyVersions)]
pub struct CompressedServerKey {
    pub(crate) integer_key: IntegerCompressedServerKey,
}

impl CompressedServerKey {
    pub fn new(keys: &ClientKey) -> Self {
        Self {
            integer_key: IntegerCompressedServerKey::new(&keys.key),
        }
    }

    pub fn into_raw_parts(
        self,
    ) -> (
        crate::integer::CompressedServerKey,
        Option<crate::integer::key_switching_key::CompressedKeySwitchingKeyMaterial>,
        Option<CompressedCompressionKey>,
        Option<CompressedDecompressionKey>,
    ) {
        self.integer_key.into_raw_parts()
    }

    pub fn from_raw_parts(
        integer_key: crate::integer::CompressedServerKey,
        cpk_key_switching_key_material: Option<
            crate::integer::key_switching_key::CompressedKeySwitchingKeyMaterial,
        >,
        compression_key: Option<CompressedCompressionKey>,
        decompression_key: Option<CompressedDecompressionKey>,
    ) -> Self {
        Self {
            integer_key: IntegerCompressedServerKey::from_raw_parts(
                integer_key,
                cpk_key_switching_key_material,
                compression_key,
                decompression_key,
            ),
        }
    }

    pub fn decompress(&self) -> ServerKey {
        ServerKey {
            key: Arc::new(self.integer_key.decompress()),
        }
    }

    #[cfg(feature = "gpu")]
    pub fn decompress_to_gpu(&self) -> CudaServerKey {
        let streams = CudaStreams::new_multi_gpu();
        synchronize_devices(streams.len() as u32);
        let cuda_key = crate::integer::gpu::CudaServerKey::decompress_from_cpu(
            &self.integer_key.key,
            &streams,
        );
        synchronize_devices(streams.len() as u32);
        CudaServerKey {
            key: Arc::new(cuda_key),
        }
    }
}

#[cfg(feature = "gpu")]
#[derive(Clone)]
pub struct CudaServerKey {
    pub(crate) key: Arc<crate::integer::gpu::CudaServerKey>,
}

#[cfg(feature = "gpu")]
impl CudaServerKey {
    pub(crate) fn message_modulus(&self) -> crate::shortint::MessageModulus {
        self.key.message_modulus
    }
}

pub enum InternalServerKey {
    Cpu(Arc<IntegerServerKey>),
    #[cfg(feature = "gpu")]
    Cuda(CudaServerKey),
}

impl From<ServerKey> for InternalServerKey {
    fn from(value: ServerKey) -> Self {
        Self::Cpu(value.key)
    }
}
#[cfg(feature = "gpu")]
impl From<CudaServerKey> for InternalServerKey {
    fn from(value: CudaServerKey) -> Self {
        Self::Cuda(value)
    }
}
