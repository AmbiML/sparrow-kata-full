//! Kata OS storage management support

#![cfg_attr(not(test), no_std)]

use kata_security_interface::kata_security_delete_key;
use kata_security_interface::kata_security_read_key;
use kata_security_interface::kata_security_write_key;
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

        // NB: must copy into KeyValueData for now
        let mut keyval = [0u8; KEY_VALUE_DATA_SIZE];
        Ok(kata_security_read_key(bundle_id, key, &mut keyval).map(|_| keyval)?)
    }
    fn write(&self, bundle_id: &str, key: &str, value: &[u8]) -> Result<(), StorageError> {
        trace!(
            "write bundle_id:{} key:{} value:{:?}",
            bundle_id,
            key,
            value
        );

        Ok(kata_security_write_key(bundle_id, key, value)?)
    }
    fn delete(&self, bundle_id: &str, key: &str) -> Result<(), StorageError> {
        trace!("delete bundle_id:{} key:{}", bundle_id, key);

        Ok(kata_security_delete_key(bundle_id, key)?)
    }
}
