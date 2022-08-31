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

#![cfg_attr(not(test), no_std)]
#![feature(build_hasher_simple_hash_one)]

use kata_os_common::camkes::seL4_CPath;
use kata_os_common::sel4_sys;
use kata_sdk_interface::error::SDKError;
use kata_sdk_interface::SDKAppId;
use kata_sdk_interface::SDKRuntimeInterface;
use kata_sdk_manager::SDKManagerError;
use kata_sdk_manager::SDKManagerInterface;
use spin::Mutex;

use sel4_sys::seL4_CPtr;

mod runtime;
use runtime::SDKRuntime;

/// Wrapper around SDKRuntime implementation. Because we have two CAmkES
/// interfaces there may be concurrent calls so we lock at this level.
pub struct KataSDKRuntime {
    runtime: Mutex<Option<SDKRuntime>>,
}
impl KataSDKRuntime {
    // Constructs a partially-initialized instance; to complete call init().
    // This is needed because we need a const fn for static setup.
    pub const fn empty() -> KataSDKRuntime {
        KataSDKRuntime {
            runtime: Mutex::new(None),
        }
    }
    // Finishes the setup started by empty():
    pub fn init(&self, endpoint: &seL4_CPath) {
        *self.runtime.lock() = Some(SDKRuntime::new(endpoint));
    }
    // Returns the bundle capacity.
    pub fn capacity(&self) -> usize { self.runtime.lock().as_ref().unwrap().capacity() }
}
// These just lock accesses and handle the necessary indirection.
impl SDKManagerInterface for KataSDKRuntime {
    fn get_endpoint(&mut self, app_id: &str) -> Result<seL4_CPtr, SDKManagerError> {
        self.runtime.lock().as_mut().unwrap().get_endpoint(app_id)
    }
    fn release_endpoint(&mut self, app_id: &str) -> Result<(), SDKManagerError> {
        self.runtime
            .lock()
            .as_mut()
            .unwrap()
            .release_endpoint(app_id)
    }
}
impl SDKRuntimeInterface for KataSDKRuntime {
    fn ping(&self, app_id: SDKAppId) -> Result<(), SDKError> {
        self.runtime.lock().as_ref().unwrap().ping(app_id)
    }
    fn log(&self, app_id: SDKAppId, msg: &str) -> Result<(), SDKError> {
        self.runtime.lock().as_ref().unwrap().log(app_id, msg)
    }
}
