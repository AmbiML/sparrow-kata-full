#![no_std]

// fake-vec-core is a stubbed out version of kata-vec-core.

use kata_ml_shared::ModelSections;

pub fn enable_interrupts(_enable: bool) {}

pub fn set_wmmu(_sections: &ModelSections) {}

pub fn run() {}

pub fn tcm_write(_offset: usize, _buf: &[u32]) {}

pub fn tcm_move(_src_offset: usize, _dest_offset: usize, _byte_length: usize) {}

pub fn clear_host_req() {}

pub fn clear_finish() {}

pub fn clear_instruction_fault() {}

pub fn clear_data_fault() {}

pub fn clear_tcm() {}

pub fn get_return_code() -> u32 {
    0
}

pub fn get_fault_register() -> u32 {
    0
}
