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

//! KataOS SDK application runtime interfaces.
//! These can also be used from KataOS services (for testing) by first
//! setting up the KATA_SDK_* data (e.g. using kata_sdk_manager_get_endpoint)

#![cfg_attr(not(test), no_std)]

pub mod error;

pub use error::SDKError;
pub use error::SDKRuntimeError;

use serde::{Deserialize, Serialize};

use sel4_sys::seL4_CPtr;
use sel4_sys::seL4_Call;
use sel4_sys::seL4_MessageInfo;
use sel4_sys::seL4_PageBits;
use sel4_sys::seL4_SetCap;

const PAGE_SIZE: usize = 1 << seL4_PageBits;

// SDKRuntime client-side state setup by ProcessManager and crt0.
// TODO(sleffler): is 1 page enough? ProcessManager should probably have
//   SDKRuntime handle this
extern "C" {
    static KATA_SDK_ENDPOINT: seL4_CPtr; // IPC connection to SDKRuntime
    static KATA_SDK_FRAME: seL4_CPtr; // RPC parameters frame
    static KATA_SDK_PARAMS: *mut u8; // Virtual address of KATA_SDK_FRAME
}

// Size of the buffers used to pass serialized data. The data structure
// sizes are bounded by the single page (4K bytes) used to marshal & unmarshal
// parameters and also by their being allocated on the stack. We balance
// these against being able to handle large amounts of data.
// XXX do sensor frames need to be passed & are they too big?

// pub for server-side logic
pub const SDKRUNTIME_REQUEST_DATA_SIZE: usize = PAGE_SIZE / 2;

/// Application identity derived from seL4 Endpoint badge setup when
/// the application is started by ProcessManager.
///
/// NB: On 32-bit platforms the kernel truncates this to 28-bits;
///     on 64-bit platforms these are 64-bits.
pub type SDKAppId = usize;

/// All RPC request must have an SDKRequestHeader at the front.
#[derive(Serialize, Deserialize)]
pub struct SDKRequestHeader {
    pub request: SDKRuntimeRequest,
}
impl SDKRequestHeader {
    pub fn new(request: SDKRuntimeRequest) -> Self { Self { request } }
}

/// All RPC responses must have an SDKReplyHeader at the front.
#[derive(Serialize, Deserialize)]
pub struct SDKReplyHeader {
    pub status: SDKRuntimeError,
}
impl SDKReplyHeader {
    pub fn new(status: SDKRuntimeError) -> Self { Self { status } }
}
impl From<SDKReplyHeader> for Result<(), SDKRuntimeError> {
    fn from(header: SDKReplyHeader) -> Result<(), SDKRuntimeError> {
        if header.status == SDKRuntimeError::SDKSuccess {
            Ok(())
        } else {
            Err(header.status)
        }
    }
}

/// SDKRuntimeRequest::Ping
#[derive(Serialize, Deserialize)]
pub struct PingRequest {}

/// SDKRuntimeRequest::Log
#[derive(Serialize, Deserialize)]
pub struct LogRequest<'a> {
    pub msg: &'a [u8],
}

#[repr(C)] // XXX needed?
#[derive(Clone, Copy, Eq, PartialEq, Serialize, Deserialize, Debug)]
pub enum SDKRuntimeRequest {
    Ping = 0, // Check runtime is alive
    Log,      // Log message: [msg: &str]
}

/// Rust interface for the SDKRuntime.
///
/// This trait defines all of the same verbs we expect to support in the component
/// interface, for both client and server, since CAmkES does not (yet) know how
/// to generate Rust bindings.
///
/// On the server side, the impl of this trait is instantiated in the component
/// as a global mutable object where the incoming calls from the CAmkES C side
/// are wrapped.
///
/// On the client side, this trait is implemented using top-level functions.
pub trait SDKRuntimeInterface {
    /// Pings the SDK runtime, going from client to server and back via CAmkES IPC.
    fn ping(&self, app_id: SDKAppId) -> Result<(), SDKError>;

    /// Logs |msg| through the system logger.
    fn log(&self, app_id: SDKAppId, msg: &str) -> Result<(), SDKError>;
}

/// Rust client-side request processing. Note there is no CAmkES stub to
/// call; everything is done here. A single page frame is attached to the
/// IPC buffer with request parameters in the first half and return values
/// in the second half. Requests must have an SDKRequestHeader serialized
/// separately from any arguments. Responses must have an SDKReplyHeader
/// included in the reply data. For the moment this uses postcard to do
/// serde work; this may change in the future (e.g. to flatbuffers).
///
/// The caller is responsible for synchronizing access to KATA_SDK_* state
/// and the IPC buffer.
//
// TODO(sleffler): this attaches the call params to the IPC; might be
//   better to keep the page(s) mapped in SDKRuntime to avoid map/unmap
//   per-RPC but that requires a vspace allocator (or somerthing special
//   purpose) and a redesign of the server side to use the endpoint badge
//   to lookup the mapped page early. Downside to a fixed mapping is it
//   limits how to handle requests w/ different-sized params (e.g. sensor
//   frame vs key-value params).
// TODO(sleffler): could send request header and reponse statatus inline.
//   This would align request arguments to the page boundary which might
//   be useful and having the reply inline would mean SDKRuntime could
//   send a meaningful error back when unable to map the page frame.
fn kata_sdk_request<'a, S: Serialize, D: Deserialize<'a>>(
    request: SDKRuntimeRequest,
    request_args: &S,
) -> Result<D, SDKRuntimeError> {
    let params_slice = unsafe { core::slice::from_raw_parts_mut(KATA_SDK_PARAMS, PAGE_SIZE) };

    // NB: server-side must do the same split
    let (request_slice, reply_slice) = params_slice.split_at_mut(SDKRUNTIME_REQUEST_DATA_SIZE);
    reply_slice.fill(0); // XXX paranoid, could zero-pad request too

    // Encode heeader with request.
    // TODO(sleffler): eliminate struct? (could add a sequence #)
    let header_size = (postcard::to_slice(&SDKRequestHeader::new(request), request_slice)
        .map_err(|_| SDKRuntimeError::SDKSerializeFailed)?)
    .len();

    // Encode arguments immediately after.
    let (_, args_slice) = request_slice.split_at_mut(header_size);
    let _ = postcard::to_slice(request_args, args_slice)
        .map_err(|_| SDKRuntimeError::SDKSerializeFailed)?;

    // Attach params & call the SDKRuntime; then wait (block) for a reply.
    unsafe {
        seL4_SetCap(0, KATA_SDK_FRAME);
        seL4_Call(KATA_SDK_ENDPOINT, seL4_MessageInfo::new(0, 0, 1, 0));
        seL4_SetCap(0, 0);
    }

    postcard::from_bytes::<D>(reply_slice).map_err(|_| SDKRuntimeError::SDKDeserializeFailed)
}

/// Rust client-side wrapper for the ping method.
#[inline]
#[allow(dead_code)]
pub fn kata_sdk_ping() -> Result<(), SDKRuntimeError> {
    let header =
        kata_sdk_request::<PingRequest, SDKReplyHeader>(SDKRuntimeRequest::Ping, &PingRequest {})?;
    header.into()
}

/// Rust client-side wrapper for the log method.
#[inline]
#[allow(dead_code)]
pub fn kata_sdk_log(msg: &str) -> Result<(), SDKRuntimeError> {
    let header = kata_sdk_request::<LogRequest, SDKReplyHeader>(
        SDKRuntimeRequest::Log,
        &LogRequest {
            msg: msg.as_bytes(),
        },
    )?;
    header.into()
}
