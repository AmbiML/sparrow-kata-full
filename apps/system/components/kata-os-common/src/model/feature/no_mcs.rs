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

// No MCS Kernel Support.

use crate::KataOsModel;
use capdl::*;

use sel4_sys::seL4_CNode;
use sel4_sys::seL4_CPtr;
use sel4_sys::seL4_CapInitThreadTCB;
use sel4_sys::seL4_Error;
use sel4_sys::seL4_Result;
use sel4_sys::seL4_SchedContext;
use sel4_sys::seL4_SchedControl;
use sel4_sys::seL4_TCB;
use sel4_sys::seL4_TCB_Configure;
use sel4_sys::seL4_TCB_SetSchedParams;
use sel4_sys::seL4_Time;
use sel4_sys::seL4_Word;

use static_assertions::assert_cfg;
assert_cfg!(not(feature = "CONFIG_KERNEL_MCS"));

impl<'a> KataOsModel<'a> {
    pub fn init_sched_ctrl(&mut self) -> seL4_Result { Ok(()) }
    pub fn init_scs(&self) -> seL4_Result { Ok(()) }
    pub fn init_fault_ep(
        &mut self,
        cdl_tcb: &CDL_Object,
    ) -> Result<(seL4_CPtr, seL4_CPtr), seL4_Error> {
        // NB: fault ep cptrs are in the configured thread's cspace.
        Ok((cdl_tcb.tcb_fault_ep(), 0 as seL4_CPtr))
    }
}

// TODO(sleffler): match syscall types
pub fn SchedControl_Configure(
    _sched_ctrl: seL4_SchedControl,
    _sel4_sc: seL4_SchedContext,
    _affinity: seL4_Word,
    _sc_budget: seL4_Time,
    _sc_period: seL4_Time,
    _sc_data: seL4_Word,
) -> seL4_Result {
    Ok(())
}

pub fn TCB_Configure(
    sel4_tcb: seL4_Word,
    sel4_fault_ep: seL4_CPtr,
    sel4_cspace_root: seL4_CNode,
    sel4_cspace_root_data: seL4_Word,
    sel4_vspace_root: seL4_CPtr,
    sel4_vspace_root_data: seL4_Word,
    ipcbuffer_addr: seL4_Word,
    sel4_ipcbuffer: seL4_CPtr,
) -> seL4_Result {
    unsafe {
        seL4_TCB_Configure(
            sel4_tcb,
            sel4_fault_ep,
            sel4_cspace_root,
            sel4_cspace_root_data,
            sel4_vspace_root,
            sel4_vspace_root_data,
            ipcbuffer_addr,
            sel4_ipcbuffer,
        )
    }
}

pub fn TCB_SchedParams(
    sel4_tcb: seL4_Word,
    mcp: seL4_Word,
    priority: seL4_Word,
    _sel4_sc: seL4_Word,
    _sel4_fault_ep: seL4_CPtr,
) -> seL4_Result {
    unsafe {
        seL4_TCB_SetSchedParams(
            sel4_tcb,
            /*authority=*/ seL4_CapInitThreadTCB as seL4_TCB,
            mcp,
            priority,
        )
    }
}

pub fn TCB_SetTimeoutEndpoint(_sel4_tcb: seL4_Word, _sel4_tempfault_ep: seL4_CPtr) -> seL4_Result {
    Ok(())
}
