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

use core::hash::BuildHasher;
use hashbrown::HashMap;
use kata_os_common::camkes::seL4_CPath;
use kata_os_common::cspace_slot::CSpaceSlot;
use kata_os_common::sel4_sys;
use kata_sdk_interface::error::SDKError;
use kata_sdk_interface::SDKAppId;
use kata_sdk_interface::SDKRuntimeInterface;
use kata_sdk_manager::SDKManagerError;
use kata_sdk_manager::SDKManagerInterface;
use log::{error, info};
use smallstr::SmallString;

use sel4_sys::seL4_CPtr;
use sel4_sys::seL4_CapRights;

// App capacity before spillover to the heap; should be the max concurrent
// started apps. Set very small because we expect, at least initially, that
// only one app at a time will be started.
const DEFAULT_APP_CAPACITY: usize = 3;

// BundleId capacity before spillover to the heap.
// TODO(sleffler): hide this; it's part of the implementation
// TODO(sleffler): shared with kata-proc-interface
const DEFAULT_BUNDLE_ID_CAPACITY: usize = 64;

type SmallId = SmallString<[u8; DEFAULT_BUNDLE_ID_CAPACITY]>;

struct SDKRuntimeState {
    id: SmallId,
}
impl SDKRuntimeState {
    pub fn new(app_id: &str) -> Self {
        Self {
            id: SmallId::from_str(app_id),
        }
    }
}

/// Kata OS SDK support for third-party applications, Rust core.
///
/// This is the actual Rust implementation of the SDK runtime component. Here's
/// where we can encapsulate all of our Rust fanciness, away from the C
/// bindings. This is the server-side implementation.
// XXX hashmap may be overkill, could use SmallVec and badge by index
pub struct SDKRuntime {
    endpoint: seL4_CPath,
    apps: HashMap<SDKAppId, SDKRuntimeState>,
}
impl SDKRuntime {
    pub fn new(endpoint: &seL4_CPath) -> Self {
        Self {
            endpoint: *endpoint,
            apps: HashMap::with_capacity(DEFAULT_APP_CAPACITY),
        }
    }

    // Calculates the badge assigned to the seL4 endpoint the client will use
    // to send requests to the SDKRuntime. This must be unique among active
    // clients but may be reused. There is no need to randomize or otherwise
    // secure this value since clients cannot forge an endpoint.
    // TODO(sleffler): is it worth doing a hash? counter is probably sufficient
    #[cfg(target_pointer_width = "32")]
    fn calculate_badge(&self, id: &SmallId) -> SDKAppId {
        (self.apps.hasher().hash_one(id) & 0x0ffffff) as SDKAppId
    }

    #[cfg(target_pointer_width = "64")]
    fn calculate_badge(&self, id: &SmallId) -> SDKAppId {
        self.apps.hasher().hash_one(id) as SDKAppId
    }

    pub fn capacity(&self) -> usize { self.apps.capacity() }
}
impl SDKManagerInterface for SDKRuntime {
    /// Returns an seL4 Endpoint capability for |app_id| to make SDKRuntime
    /// requests..Without a registered endpoint all requests will fail.
    /// first calling kata_sdk_manager_get_endpoint().
    fn get_endpoint(&mut self, app_id: &str) -> Result<seL4_CPtr, SDKManagerError> {
        let badge = self.calculate_badge(&SmallId::from_str(app_id));

        // Mint a badged endpoint for the client to talk to us.
        let mut slot = CSpaceSlot::new();
        slot.mint_to(
            self.endpoint.0,
            self.endpoint.1,
            self.endpoint.2 as u8,
            seL4_CapRights::new(
                /*grant_reply=*/ 1,
                /*grant=*/ 1, // NB: to send frame with RPC params
                /*read=*/ 0, /*write=*/ 1,
            ),
            badge,
        )
        .map_err(|_| SDKManagerError::SmGetEndpointFailed)?;

        // Create the entry & return the endpoint capability.
        assert!(self
            .apps
            .insert(badge, SDKRuntimeState::new(app_id))
            .is_none());
        Ok(slot.release())
    }

    /// Releases |app_id| state. No future requests may be made without
    /// first calling kata_sdk_manager_get_endpoint().
    fn release_endpoint(&mut self, app_id: &str) -> Result<(), SDKManagerError> {
        let badge = self.calculate_badge(&SmallId::from_str(app_id));
        let _ = self.apps.remove(&badge);
        Ok(())
    }
}
impl SDKRuntimeInterface for SDKRuntime {
    /// Pings the SDK runtime, going from client to server and back via CAmkES IPC.
    fn ping(&self, app_id: SDKAppId) -> Result<(), SDKError> {
        match self.apps.get(&app_id) {
            Some(_) => Ok(()),
            None => {
                // XXX potential console spammage/DOS
                error!("No entry for app_id {:x}", app_id);
                Err(SDKError::InvalidBadge)
            }
        }
    }

    /// Logs |msg| through the system logger.
    fn log(&self, app_id: SDKAppId, msg: &str) -> Result<(), SDKError> {
        match self.apps.get(&app_id) {
            Some(app) => {
                info!("[{}] {}", app.id, msg);
                Ok(())
            }
            None => Err(SDKError::InvalidBadge),
        }
    }
}
