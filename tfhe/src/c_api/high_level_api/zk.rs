use super::utils::*;
use crate::c_api::high_level_api::config::Config;
use crate::c_api::utils::get_ref_checked;
use crate::zk::Compressible;
use std::ffi::c_int;

#[repr(C)]
#[derive(Copy, Clone)]
pub enum ZkComputeLoad {
    ZkComputeLoadProof,
    ZkComputeLoadVerify,
}

impl From<ZkComputeLoad> for crate::zk::ZkComputeLoad {
    fn from(value: ZkComputeLoad) -> Self {
        match value {
            ZkComputeLoad::ZkComputeLoadProof => Self::Proof,
            ZkComputeLoad::ZkComputeLoadVerify => Self::Verify,
        }
    }
}

pub struct CompactPkePublicParams(pub(crate) crate::core_crypto::entities::CompactPkePublicParams);
impl_destroy_on_type!(CompactPkePublicParams);

/// Serializes the public params
///
/// If compress is true, the data will be compressed (less serialized bytes), however, this makes
/// the deserialization process slower.
#[no_mangle]
pub unsafe extern "C" fn compact_pke_public_params_serialize(
    sself: *const CompactPkePublicParams,
    compress: bool,
    result: *mut crate::c_api::buffer::DynamicBuffer,
) -> ::std::os::raw::c_int {
    crate::c_api::utils::catch_panic(|| {
        crate::c_api::utils::check_ptr_is_non_null_and_aligned(result).unwrap();

        let wrapper = crate::c_api::utils::get_ref_checked(sself).unwrap();

        let buffer = if compress {
            bincode::serialize(&wrapper.0.compress()).unwrap()
        } else {
            bincode::serialize(&wrapper.0).unwrap()
        };

        *result = buffer.into();
    })
}

/// Deserializes the public params
///
/// If the data comes from compressed public params, then `is_compressed` must be true.
#[no_mangle]
pub unsafe extern "C" fn compact_pke_public_params_deserialize(
    buffer_view: crate::c_api::buffer::DynamicBufferView,
    result: *mut *mut CompactPkePublicParams,
) -> ::std::os::raw::c_int {
    crate::c_api::utils::catch_panic(|| {
        crate::c_api::utils::check_ptr_is_non_null_and_aligned(result).unwrap();

        *result = std::ptr::null_mut();

        let deserialized = bincode::deserialize(buffer_view.as_slice()).unwrap();

        let heap_allocated_object = Box::new(CompactPkePublicParams(deserialized));

        *result = Box::into_raw(heap_allocated_object);
    })
}

/// Serializes the public params
///
/// If compress is true, the data will be compressed (less serialized bytes), however, this makes
/// the deserialization process slower.
#[no_mangle]
pub unsafe extern "C" fn compact_pke_public_params_safe_serialize(
    sself: *const CompactPkePublicParams,
    compress: bool,
    serialized_size_limit: u64,
    result: *mut crate::c_api::buffer::DynamicBuffer,
) -> ::std::os::raw::c_int {
    crate::c_api::utils::catch_panic(|| {
        crate::c_api::utils::check_ptr_is_non_null_and_aligned(result).unwrap();

        let wrapper = crate::c_api::utils::get_ref_checked(sself).unwrap();

        let mut buffer = Vec::new();
        if compress {
            crate::safe_serialization::SerializationConfig::new(serialized_size_limit)
                .serialize_into(&wrapper.0.compress(), &mut buffer)
                .unwrap();
        } else {
            crate::safe_serialization::SerializationConfig::new(serialized_size_limit)
                .serialize_into(&wrapper.0, &mut buffer)
                .unwrap();
        };

        *result = buffer.into();
    })
}

/// Deserializes the public params
///
/// If the data comes from compressed public params, then `is_compressed` must be true.
#[no_mangle]
pub unsafe extern "C" fn compact_pke_public_params_safe_deserialize(
    buffer_view: crate::c_api::buffer::DynamicBufferView,
    serialized_size_limit: u64,
    result: *mut *mut CompactPkePublicParams,
) -> ::std::os::raw::c_int {
    crate::c_api::utils::catch_panic(|| {
        crate::c_api::utils::check_ptr_is_non_null_and_aligned(result).unwrap();

        *result = std::ptr::null_mut();

        let buffer_view: &[u8] = buffer_view.as_slice();

        let deserialized =
            crate::safe_serialization::DeserializationConfig::new(serialized_size_limit)
                .disable_conformance()
                .deserialize_from(buffer_view)
                .unwrap();

        let heap_allocated_object = Box::new(CompactPkePublicParams(deserialized));

        *result = Box::into_raw(heap_allocated_object);
    })
}

pub struct CompactPkeCrs(pub(crate) crate::core_crypto::entities::CompactPkeCrs);

impl_destroy_on_type!(CompactPkeCrs);

#[no_mangle]
pub unsafe extern "C" fn compact_pke_crs_from_config(
    config: *const Config,
    max_num_bits: usize,
    out_result: *mut *mut CompactPkeCrs,
) -> c_int {
    crate::c_api::utils::catch_panic(|| {
        let config = get_ref_checked(config).unwrap();

        let crs = crate::core_crypto::entities::CompactPkeCrs::from_config(config.0, max_num_bits)
            .unwrap();

        *out_result = Box::into_raw(Box::new(CompactPkeCrs(crs)));
    })
}

#[no_mangle]
pub unsafe extern "C" fn compact_pke_crs_public_params(
    crs: *const CompactPkeCrs,
    out_public_params: *mut *mut CompactPkePublicParams,
) -> c_int {
    crate::c_api::utils::catch_panic(|| {
        let crs = get_ref_checked(crs).unwrap();

        *out_public_params = Box::into_raw(Box::new(CompactPkePublicParams(
            crs.0.public_params().clone(),
        )));
    })
}
