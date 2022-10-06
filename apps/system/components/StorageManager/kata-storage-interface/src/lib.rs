//! Kata OS storage management support

#![cfg_attr(not(test), no_std)]

use core::str;
use cstr_core::CString;
use kata_security_interface::SecurityRequestError;
use postcard;

// TODO(sleffler): temp constraint on value part of key-value pairs
pub const KEY_VALUE_DATA_SIZE: usize = 100;
pub type KeyValueData = [u8; KEY_VALUE_DATA_SIZE];

// NB: struct's marked repr(C) are processed by cbindgen to get a .h file
//   used in camkes C interfaces.

#[derive(Debug, Eq, PartialEq)]
pub enum StorageError {
    BundleNotFound = 0,
    KeyNotFound,
    KeyInvalid,
    ValueInvalid,
    SerializeFailed,
    UnknownSecurityError,
    // Generic errors.
    ReadFailed,
    WriteFailed,
    DeleteFailed,
}

impl From<SecurityRequestError> for StorageError {
    fn from(err: SecurityRequestError) -> StorageError {
        match err {
            SecurityRequestError::SreBundleNotFound => StorageError::BundleNotFound,
            SecurityRequestError::SreKeyNotFound => StorageError::KeyNotFound,
            SecurityRequestError::SreValueInvalid => StorageError::ValueInvalid,
            SecurityRequestError::SreKeyInvalid => StorageError::KeyInvalid,
            SecurityRequestError::SreSerializeFailed => StorageError::SerializeFailed,
            SecurityRequestError::SreReadFailed => StorageError::ReadFailed,
            SecurityRequestError::SreWriteFailed => StorageError::WriteFailed,
            SecurityRequestError::SreDeleteFailed => StorageError::DeleteFailed,
            _ => StorageError::UnknownSecurityError, // NB: cannot happen
        }
    }
}

impl From<postcard::Error> for StorageError {
    fn from(_err: postcard::Error) -> StorageError {
        StorageError::SerializeFailed
    }
}

pub trait StorageManagerInterface {
    fn read(&self, bundle_id: &str, key: &str) -> Result<KeyValueData, StorageError>;
    fn write(&self, bundle_id: &str, key: &str, value: &[u8]) -> Result<(), StorageError>;
    fn delete(&self, bundle_id: &str, key: &str) -> Result<(), StorageError>;
}

// Public version of StorageError presented over rpc interface.
// This is needed because the enum is exported to C users and needs to
// be unique from other enum's.
// TODO(sleffler): switch to single generic error space ala absl::StatusCode
#[repr(C)]
#[derive(Debug, Eq, PartialEq)]
pub enum StorageManagerError {
    SmeSuccess = 0,
    SmeBundleIdInvalid,
    SmeBundleNotFound,
    SmeKeyNotFound,
    SmeValueInvalid,
    SmeKeyInvalid,
    // Generic errors.
    SmeReadFailed,
    SmeWriteFailed,
    SmeDeleteFailed,
    SmeUnknownError,
}

impl From<StorageError> for StorageManagerError {
    fn from(err: StorageError) -> StorageManagerError {
        match err {
            StorageError::BundleNotFound => StorageManagerError::SmeBundleNotFound,
            StorageError::KeyNotFound => StorageManagerError::SmeKeyNotFound,
            StorageError::KeyInvalid => StorageManagerError::SmeKeyInvalid,
            StorageError::ValueInvalid => StorageManagerError::SmeValueInvalid,
            StorageError::ReadFailed => StorageManagerError::SmeReadFailed,
            StorageError::WriteFailed => StorageManagerError::SmeWriteFailed,
            StorageError::DeleteFailed => StorageManagerError::SmeDeleteFailed,
            _ => StorageManagerError::SmeUnknownError,
        }
    }
}

impl From<Result<(), StorageError>> for StorageManagerError {
    fn from(result: Result<(), StorageError>) -> StorageManagerError {
        result.map_or_else(
            |e| StorageManagerError::from(e),
            |_v| StorageManagerError::SmeSuccess,
        )
    }
}

impl From<cstr_core::NulError> for StorageManagerError {
    fn from(_err: cstr_core::NulError) -> StorageManagerError {
        StorageManagerError::SmeKeyInvalid
    }
}

impl From<StorageManagerError> for Result<(), StorageManagerError> {
    fn from(err: StorageManagerError) -> Result<(), StorageManagerError> {
        if err == StorageManagerError::SmeSuccess {
            Ok(())
        } else {
            Err(err)
        }
    }
}

#[inline]
#[allow(dead_code)]
pub fn kata_storage_delete(key: &str) -> Result<(), StorageManagerError> {
    // NB: this assumes the StorageManager component is named "storage".
    extern "C" {
        pub fn storage_delete(c_key: *const cstr_core::c_char) -> StorageManagerError;
    }
    let cstr = CString::new(key)?;
    unsafe { storage_delete(cstr.as_ptr()) }.into()
}

#[inline]
#[allow(dead_code)]
pub fn kata_storage_read(key: &str) -> Result<KeyValueData, StorageManagerError> {
    extern "C" {
        fn storage_read(
            c_key: *const cstr_core::c_char,
            c_raw_value: *mut KeyValueData,
        ) -> StorageManagerError;
    }
    let cstr = CString::new(key)?;
    let value = &mut [0u8; KEY_VALUE_DATA_SIZE];
    match unsafe { storage_read(cstr.as_ptr(), value as *mut _) } {
        StorageManagerError::SmeSuccess => Ok(*value),
        status => Err(status),
    }
}

#[inline]
#[allow(dead_code)]
pub fn kata_storage_write(key: &str, value: &[u8]) -> Result<(), StorageManagerError> {
    extern "C" {
        fn storage_write(
            c_key: *const cstr_core::c_char,
            c_raw_value_len: usize,
            c_raw_value: *const u8,
        ) -> StorageManagerError;
    }
    let cstr = CString::new(key)?;
    unsafe { storage_write(cstr.as_ptr(), value.len(), value.as_ptr()) }.into()
}
