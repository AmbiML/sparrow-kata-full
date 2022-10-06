// Copyright 2022 Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     https://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Kata OS security coordinator support

#![cfg_attr(not(test), no_std)]
// NB: "error[E0658]: trait bounds other than `Sized` on const fn parameters are unstable"
#![feature(const_fn_trait_bound)]

extern crate alloc;
use alloc::boxed::Box;
use alloc::string::String;
use kata_memory_interface::ObjDescBundle;
use kata_security_interface::SecurityCoordinatorInterface;
use kata_security_interface::SecurityRequestError;
use kata_security_interface::KeyValueData;

#[cfg(all(feature = "fake", feature = "sel4"))]
compile_error!("features \"fake\" and \"sel4\" are mutually exclusive");

#[cfg_attr(feature = "sel4", path = "impl.rs")]
#[cfg_attr(feature = "fake", path = "fakeimpl/mod.rs")]
mod platform;
pub use platform::KataSecurityCoordinatorInterface;

#[cfg(not(test))]
pub static mut KATA_SECURITY: KataSecurityCoordinator = KataSecurityCoordinator::empty();

// KataSecurityCoordinator bundles an instance of the SecurityCoordinator that operates
// on KataOS interfaces. There is a two-step dance to setup an instance because we want
// KATA_SECURITY static.
// NB: no locking is done; we assume the caller/user is single-threaded
pub struct KataSecurityCoordinator {
    manager: Option<Box<dyn SecurityCoordinatorInterface + Sync>>,
}
impl KataSecurityCoordinator {
    // Constructs a partially-initialized instance; to complete call init().
    // This is needed because we need a const fn for static setup.
    const fn empty() -> KataSecurityCoordinator { KataSecurityCoordinator { manager: None } }

    pub fn init(&mut self) {
        self.manager = Some(Box::new(KataSecurityCoordinatorInterface::new()));
    }
}
impl SecurityCoordinatorInterface for KataSecurityCoordinator {
    fn install(&mut self, pkg_contents: &ObjDescBundle) -> Result<String, SecurityRequestError> {
        self.manager.as_mut().unwrap().install(pkg_contents)
    }
    fn uninstall(&mut self, bundle_id: &str) -> Result<(), SecurityRequestError> {
        self.manager.as_mut().unwrap().uninstall(bundle_id)
    }
    fn size_buffer(&self, bundle_id: &str) -> Result<usize, SecurityRequestError> {
        self.manager.as_ref().unwrap().size_buffer(bundle_id)
    }
    fn get_manifest(&self, bundle_id: &str) -> Result<String, SecurityRequestError> {
        self.manager.as_ref().unwrap().get_manifest(bundle_id)
    }
    fn load_application(&self, bundle_id: &str) -> Result<ObjDescBundle, SecurityRequestError> {
        self.manager.as_ref().unwrap().load_application(bundle_id)
    }
    fn load_model(
        &self,
        bundle_id: &str,
        model_id: &str,
    ) -> Result<ObjDescBundle, SecurityRequestError> {
        self.manager
            .as_ref()
            .unwrap()
            .load_model(bundle_id, model_id)
    }
    fn read_key(&self, bundle_id: &str, key: &str) -> Result<&KeyValueData, SecurityRequestError> {
        self.manager.as_ref().unwrap().read_key(bundle_id, key)
    }
    fn write_key(
        &mut self,
        bundle_id: &str,
        key: &str,
        value: &KeyValueData,
    ) -> Result<(), SecurityRequestError> {
        self.manager
            .as_mut()
            .unwrap()
            .write_key(bundle_id, key, value)
    }
    fn delete_key(&mut self, bundle_id: &str, key: &str) -> Result<(), SecurityRequestError> {
        self.manager.as_mut().unwrap().delete_key(bundle_id, key)
    }
    fn test_mailbox(&mut self) -> Result<(), SecurityRequestError> {
        self.manager.as_mut().unwrap().test_mailbox()
    }
}
