//! Kata OS storage management support

#![cfg_attr(not(test), no_std)]

use kata_security_interface::kata_security_request;
use kata_security_interface::DeleteKeyRequest;
use kata_security_interface::ReadKeyRequest;
use kata_security_interface::SecurityRequest;
use kata_security_interface::WriteKeyRequest;
use kata_security_interface::SECURITY_REPLY_DATA_SIZE;
use kata_storage_interface::StorageError;
use kata_storage_interface::StorageManagerInterface;
use kata_storage_interface::{KeyValueData, KEY_VALUE_DATA_SIZE};
use log::trace;

#[cfg(not(test))]
pub static mut KATA_STORAGE: KataStorageManager = KataStorageManager {};

pub struct KataStorageManager;
impl StorageManagerInterface for KataStorageManager {
    fn read(&self, bundle_id: &str, key: &str) -> Result<KeyValueData, StorageError> {
        trace!("read bundle_id:{} key:{}", bundle_id, key);

        // Send request to Security Core via SecurityCoordinator
        let result = &mut [0u8; SECURITY_REPLY_DATA_SIZE];
        kata_security_request(
            SecurityRequest::SrReadKey,
            &ReadKeyRequest {
                bundle_id: bundle_id,
                key: key,
            },
            result,
        )?;
        // NB: must copy into KeyValueData for now
        let mut keyval = [0u8; KEY_VALUE_DATA_SIZE];
        keyval.copy_from_slice(&result[..KEY_VALUE_DATA_SIZE]);
        Ok(keyval)
    }
    fn write(&self, bundle_id: &str, key: &str, value: &[u8]) -> Result<(), StorageError> {
        trace!(
            "write bundle_id:{} key:{} value:{:?}",
            bundle_id,
            key,
            value
        );

        // Send request to Security Core via SecurityCoordinator
        let result = &mut [0u8; SECURITY_REPLY_DATA_SIZE];
        kata_security_request(
            SecurityRequest::SrWriteKey,
            &WriteKeyRequest {
                bundle_id: bundle_id,
                key: key,
                value: value,
            },
            result,
        )?;
        Ok(())
    }
    fn delete(&self, bundle_id: &str, key: &str) -> Result<(), StorageError> {
        trace!("delete bundle_id:{} key:{}", bundle_id, key);

        // Send request to Security Core via SecurityCoordinator
        let result = &mut [0u8; SECURITY_REPLY_DATA_SIZE];
        kata_security_request(
            SecurityRequest::SrDeleteKey,
            &DeleteKeyRequest {
                bundle_id: bundle_id,
                key: key,
            },
            result,
        )?;
        Ok(())
    }
}
