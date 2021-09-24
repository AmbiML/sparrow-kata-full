//! Kata OS storage management support

#![cfg_attr(not(test), no_std)]

extern crate alloc;
use alloc::string::String;
use kata_security_interface::kata_security_request;
use kata_security_interface::DeleteKeyRequest;
use kata_security_interface::ReadKeyRequest;
use kata_security_interface::SecurityRequest;
use kata_security_interface::WriteKeyRequest;
use kata_security_interface::SECURITY_REPLY_DATA_SIZE;
use kata_security_interface::SECURITY_REQUEST_DATA_SIZE;
use kata_storage_interface::StorageError;
use kata_storage_interface::StorageManagerInterface;
use kata_storage_interface::{KeyValueData, KEY_VALUE_DATA_SIZE};
use log::trace;
use postcard;

// NB: KATA_STORAGE cannot be used before setup is completed with a call to init()
#[cfg(not(test))]
pub static mut KATA_STORAGE: KataStorageManager = KataStorageManager {};

// KataStorageManager bundles an instance of the StorageManager that operates
// on KataOS interfaces. There is a two-step dance to setup an instance because
// we want KATA_STORAGE static and there is no const Box::new variant.
pub struct KataStorageManager;
impl StorageManagerInterface for KataStorageManager {
    fn read(&self, bundle_id: &str, key: &str) -> Result<KeyValueData, StorageError> {
        trace!("read bundle_id:{} key:{}", bundle_id, key);

        // Send request to Security Core via SecurityCoordinator
        let mut request = [0u8; SECURITY_REQUEST_DATA_SIZE];
        let _ = postcard::to_slice(
            &ReadKeyRequest {
                bundle_id: String::from(bundle_id),
                key: String::from(key),
            },
            &mut request[..],
        )?;
        let result = &mut [0u8; SECURITY_REPLY_DATA_SIZE];
        let _ = kata_security_request(SecurityRequest::SrReadKey, &request, result)?;
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
        let mut request = [0u8; SECURITY_REQUEST_DATA_SIZE];
        let _ = postcard::to_slice(
            &WriteKeyRequest {
                bundle_id: String::from(bundle_id),
                key: String::from(key),
                value: value,
            },
            &mut request[..],
        )?;
        let result = &mut [0u8; SECURITY_REPLY_DATA_SIZE];
        kata_security_request(SecurityRequest::SrWriteKey, &request, result)?;
        Ok(())
    }
    fn delete(&self, bundle_id: &str, key: &str) -> Result<(), StorageError> {
        trace!("delete bundle_id:{} key:{}", bundle_id, key);

        // Send request to Security Core via SecurityCoordinator
        let mut request = [0u8; SECURITY_REQUEST_DATA_SIZE];
        let _ = postcard::to_slice(
            &DeleteKeyRequest {
                bundle_id: String::from(bundle_id),
                key: String::from(key),
            },
            &mut request[..],
        )?;
        let result = &mut [0u8; SECURITY_REPLY_DATA_SIZE];
        kata_security_request(SecurityRequest::SrDeleteKey, &request, result)?;
        Ok(())
    }
}
